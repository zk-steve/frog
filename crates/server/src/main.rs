use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use clap::{Parser, Subcommand};
use frog_adapter::in_memory::session::SessionInMemoryRepository;
use frog_adapter::in_memory::state::InMemoryState;
use frog_common::cli_args::CliArgs;
use frog_common::kill_signals;
use frog_common::loggers::telemetry::init_telemetry;
use frog_core::ports::session::SessionPort;
use frog_server::app_state::AppState;
use frog_server::options::Options;
use frog_server::routes::routes;
use frog_server::services::session::SessionService;
use graphile_worker::WorkerUtils;
use opentelemetry::global;
use phantom::phantom_zone::{Crs, Param};
use phantom::I_2P_60;
use phantom_zone_evaluator::boolean::fhew::prelude::{DecompositionParam, Modulus};
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

    let in_memory_session_state = Arc::new(RwLock::new(InMemoryState::default()));
    let session_port: Arc<dyn SessionPort + Send + Sync> =
        Arc::new(SessionInMemoryRepository::new(in_memory_session_state));

    // Parse and pad the crs seed from config
    let mut crs_seed = [0u8; 32]; // Create a zero-filled array of 32 bytes
    let bytes = options.phantom_server.crs_seed.as_bytes(); // Get the byte representation of the string
    let len = bytes.len().min(32);
    crs_seed[..len].copy_from_slice(&bytes[..len]);

    let crs = Crs::new(crs_seed);

    let phantom_param = Param {
        param: I_2P_60,
        ring_packing_modulus: Some(Modulus::Prime(2305843009213554689)),
        ring_packing_auto_decomposition_param: DecompositionParam {
            log_base: 20,
            level: 1,
        },
    };

    let worker_utils = Arc::new(WorkerUtils::new(pg_pool, options.worker.schema.clone()));
    let session_service = Arc::new(SessionService::new(
        session_port,
        phantom_param,
        crs,
        options.phantom_server.participant_number,
    ));
    session_service.create().await.unwrap();

    let routes = routes(AppState::new(worker_utils, session_service)).layer((
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
