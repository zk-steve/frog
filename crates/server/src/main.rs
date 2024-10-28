use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use clap::{Parser, Subcommand};
use frog_adapter::in_memory::game::GameInMemoryRepository;
use frog_adapter::in_memory::game_player::GamePlayerInMemoryRepository;
use frog_adapter::in_memory::state::InMemoryState;
use frog_common::cli_args::CliArgs;
use frog_common::kill_signals;
use frog_common::loggers::telemetry::init_telemetry;
use frog_core::ports::game::GamePort;
use frog_core::ports::game_player::GamePlayerPort;
use frog_server::app_state::AppState;
use frog_server::options::Options;
use frog_server::routes::routes;
use frog_server::services::game::GameService;
use frog_server::services::game_player::GamePlayerService;
use graphile_worker::WorkerUtils;
use opentelemetry::global;
use sqlx::postgres::PgConnectOptions;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    let options: Options = CliArgs::default_run_or_get_options(env!("APP_VERSION"));

    init_telemetry(
        options.service_name.as_str(),
        options.exporter_endpoint.as_str(),
        options.log.level.as_str(),
    );

    let server = tokio::spawn(serve(options));

    // Wait for the server to finish shutting down
    tokio::try_join!(server).expect("Failed to run server");

    global::shutdown_tracer_provider();
    info!("Shutdown successfully!");
}

/// Frog Server.
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
    /// Config file
    #[arg(short, long, default_value = "config/00-default.toml")]
    config_path: Vec<String>,
    /// Print version
    #[clap(short, long)]
    version: bool,
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    /// Print config
    Config,
}

pub async fn serve(options: Options) {
    let pg_options = PgConnectOptions::from_str(&options.pg.url).unwrap();

    let pg_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(options.pg.max_size)
        .connect_with(pg_options)
        .await
        .unwrap();

    let in_memory_game_state = Arc::new(RwLock::new(InMemoryState::default()));
    let game_port: Arc<dyn GamePort + Send + Sync> =
        Arc::new(GameInMemoryRepository::new(in_memory_game_state.clone()));
    let game_player_port: Arc<dyn GamePlayerPort + Send + Sync> = Arc::new(
        GamePlayerInMemoryRepository::new(in_memory_game_state.clone()),
    );

    let worker_utils = Arc::new(WorkerUtils::new(pg_pool, options.worker.schema.clone()));
    let game_service = Arc::new(GameService::new(game_port));
    let game_player_service = Arc::new(GamePlayerService::new(game_player_port));

    let routes = routes(AppState::new(
        worker_utils,
        game_service,
        game_player_service,
    ))
    .layer((
        TraceLayer::new_for_http(),
        // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
        // requests don't hang forever.
        TimeoutLayer::new(Duration::from_secs(10)),
    ));

    let endpoint = format!("{}:{}", options.server.url.as_str(), options.server.port);
    let listener = tokio::net::TcpListener::bind(endpoint.clone())
        .await
        .unwrap();
    info!("listening on http://{}", endpoint);
    axum::serve(listener, routes)
        .with_graceful_shutdown(kill_signals::wait_for_kill_signals())
        .await
        .unwrap();
}
