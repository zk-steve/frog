use std::sync::Arc;
use std::time::Duration;

use clap::{Parser, Subcommand};
use deadpool_diesel::postgres::Pool;
use deadpool_diesel::{Manager, Runtime};
use diesel_migrations::MigrationHarness;
use frog_adapter::postgres::session_db::{SessionDBRepository, MIGRATIONS};
use frog_adapter::worker::WorkerAdapter;
use frog_common::cli_args::CliArgs;
use frog_common::kill_signals;
use frog_common::loggers::telemetry::init_telemetry;
use frog_core::ports::session::SessionPort;
use frog_core::ports::worker::WorkerPort;
use frog_server::app_state::AppState;
use frog_server::options::Options;
use frog_server::routes::routes;
use frog_server::services::session::SessionService;
use opentelemetry::global;
use phantom::crs::Crs;
use phantom::param::Param;
use phantom::utils::I_2P_60;
use phantom_zone_evaluator::boolean::fhew::prelude::{DecompositionParam, Modulus};
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
    let manager = Manager::new(&options.pg.url, Runtime::Tokio1);
    let pool = Pool::builder(manager)
        .max_size(options.pg.max_size as usize)
        .build()
        .unwrap();

    // Migration the database
    let conn = pool.get().await.unwrap();
    let _ = conn
        .interact(|connection| {
            let result = MigrationHarness::run_pending_migrations(connection, MIGRATIONS);
            match result {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        })
        .await;

    let session_port: Arc<dyn SessionPort + Send + Sync> = Arc::new(SessionDBRepository::new(pool));

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

    let worker_adapter: Arc<dyn WorkerPort + Send + Sync> = Arc::new(
        WorkerAdapter::new(
            &options.pg.url,
            options.pg.max_size,
            options.worker.schema.clone(),
        )
        .await,
    );

    let session_service = Arc::new(SessionService::new(
        session_port,
        phantom_param,
        crs,
        options.phantom_server.participant_number,
        worker_adapter,
    ));
    session_service.delete().await.unwrap();
    session_service.create().await.unwrap();

    let routes = routes(AppState::new(session_service)).layer((
        TraceLayer::new_for_http(),
        // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
        // requests don't hang forever.
        TimeoutLayer::new(Duration::from_secs(5 * 60)),
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
