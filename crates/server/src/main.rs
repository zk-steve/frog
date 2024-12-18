use std::sync::Arc;
use std::time::Duration;

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
use phantom::utils::{pad_seed_to_32_bytes, I_2P_60};
use phantom_zone_evaluator::boolean::fhew::prelude::{DecompositionParam, Modulus};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    // Parse CLI arguments and load options.
    let options: Options = CliArgs::default_run_or_get_options(env!("APP_VERSION"));

    // Initialize telemetry for distributed tracing and logging.
    init_telemetry(
        options.service_name.as_str(),
        options.exporter_endpoint.as_str(),
        options.log.level.as_str(),
    );

    // Start the server as a separate asynchronous task.
    let server_task = tokio::spawn(serve(options));

    // Wait for the server to finish shutting down
    tokio::try_join!(server_task).expect("Failed to run server");

    // Shutdown the tracing provider before exiting.
    global::shutdown_tracer_provider();

    info!("Shutdown successfully!");
}

/// Frog Server.
pub async fn serve(options: Options) {
    // Initialize the database connection pool.
    let manager = Manager::new(&options.pg.url, Runtime::Tokio1);
    let pool = Pool::builder(manager)
        .max_size(options.pg.max_size as usize)
        .build()
        .expect("Failed to create database pool");

    // Run database migrations at startup.
    let conn = pool.get().await.unwrap();
    conn.interact(|connection| {
        let result = MigrationHarness::run_pending_migrations(connection, MIGRATIONS);
        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    })
    .await
    .expect("Failed to connect to the database")
    .expect("Failed to migrate the database");

    // Parse CRS seed from configuration and ensure it's 32 bytes long.
    let crs_seed = pad_seed_to_32_bytes(options.phantom_server.crs_seed.as_bytes());

    // Initialize the CRS and Phantom client parameters.
    let crs = Crs::new(crs_seed);

    // Set up the Phantom parameters for encryption.
    let phantom_param = initialize_phantom_parameters();

    // Initialize the SessionPort implementation (database repository).
    let session_port: Arc<dyn SessionPort + Send + Sync> = Arc::new(SessionDBRepository::new(pool));

    // Set up the worker adapter for background task handling.
    let worker_adapter: Arc<dyn WorkerPort + Send + Sync> = Arc::new(
        WorkerAdapter::new(
            &options.pg.url,
            options.pg.max_size,
            options.worker.schema.clone(),
        )
        .await,
    );

    // Create and initialize the SessionService, which coordinates session operations.
    let session_service = Arc::new(SessionService::new(
        session_port,
        phantom_param,
        crs,
        options.phantom_server.participant_number,
        worker_adapter,
    ));

    // Reset the session.
    session_service.delete().await.unwrap();
    session_service.create().await.unwrap();

    // Configure HTTP routes with middleware for tracing and request timeout.
    let routes = routes(AppState::new(session_service)).layer((
        TraceLayer::new_for_http(),
        TimeoutLayer::new(Duration::from_secs(5 * 60)), // Ensure requests don't hang indefinitely.
    ));

    // Start the HTTP server and listen for incoming requests.
    let endpoint = format!("{}:{}", options.server.url.as_str(), options.server.port);
    let listener = tokio::net::TcpListener::bind(&endpoint)
        .await
        .unwrap_or_else(|e| {
            panic!("Failed to bind server to {}: {}", endpoint, e);
        });

    info!("Listening on http://{}", endpoint);

    axum::serve(listener, routes)
        .with_graceful_shutdown(kill_signals::wait_for_kill_signals())
        .await
        .unwrap();
}

/// Configures the encryption parameters for the Phantom library.
///
/// # Returns
/// A `Param` structure containing the encryption parameters.
fn initialize_phantom_parameters() -> Param {
    Param {
        param: I_2P_60,
        ring_packing_modulus: Some(Modulus::Prime(2305843009213554689)),
        ring_packing_auto_decomposition_param: DecompositionParam {
            log_base: 20,
            level: 1,
        },
    }
}
