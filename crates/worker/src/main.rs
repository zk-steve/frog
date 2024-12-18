mod options;
mod routes;

use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use clap::{Parser, Subcommand};
use deadpool_diesel::postgres::Pool;
use deadpool_diesel::{Manager, Runtime};
use frog_adapter::postgres::session_db::SessionDBRepository;
use frog_common::cli_args::CliArgs;
use frog_common::kill_signals;
use frog_common::loggers::telemetry::init_telemetry;
use frog_core::ports::session::SessionPort;
use frog_worker::app_state::AppState;
use frog_worker::services::session::SessionService;
use frog_worker::workers::bs_key_shares::BsKeySharesWorker;
use frog_worker::workers::compute_function::ComputeFunctionWorker;
use graphile_worker::WorkerOptions;
use opentelemetry::global;
use sqlx::postgres::PgConnectOptions;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::options::Options;
use crate::routes::routes;

#[tokio::main]
async fn main() {
    // Parse command-line arguments or configuration options
    let options: Options = CliArgs::default_run_or_get_options(env!("APP_VERSION"));

    // Initialize telemetry for structured logging and monitoring
    init_telemetry(
        options.service_name.as_str(),
        options.exporter_endpoint.as_str(),
        options.log.level.as_str(),
    );

    // Start the HTTP server
    let server = tokio::spawn(serve(options.clone()));

    // Run background workers
    run_workers(options).await;

    // Wait for the server task to complete
    tokio::try_join!(server).expect("Failed to run server");

    // Cleanly shut down the OpenTelemetry tracer provider
    global::shutdown_tracer_provider();
    info!("Shutdown successfully!");
}

/// Represents command-line arguments for the Frog Worker.
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path(s) to configuration file(s).
    #[arg(short, long, default_value = "config/00-default.toml")]
    config_path: Vec<String>,

    /// Print the version.
    #[clap(short, long)]
    version: bool,
}

/// Subcommands supported by the Frog Worker application.
#[derive(Subcommand, Clone, Debug)]
enum Commands {
    /// Print the current configuration.
    Config,
}

/// Starts the HTTP server for the Frog Worker.
pub async fn serve(options: Options) {
    // Create the Axum routes and add middleware layers for tracing and timeouts
    let routes = routes().layer((
        TraceLayer::new_for_http(),
        TimeoutLayer::new(Duration::from_secs(10)), // Prevents requests from hanging indefinitely
    ));

    // Bind the server to the specified endpoint and start listening
    let endpoint = format!("{}:{}", options.server.url, options.server.port);
    let listener = tokio::net::TcpListener::bind(endpoint.clone())
        .await
        .expect("Failed to bind to the server endpoint");
    info!("Listening on http://{}", endpoint);

    // Serve the routes and gracefully shut down on kill signals
    axum::serve(listener, routes)
        .with_graceful_shutdown(kill_signals::wait_for_kill_signals())
        .await
        .expect("Server encountered an error");
}

/// Configures and runs the background workers for the Frog Worker.
pub async fn run_workers(options: Options) {
    // Configure PostgreSQL connection options
    let pg_options =
        PgConnectOptions::from_str(&options.pg.url).expect("Invalid PostgreSQL connection string");

    // Initialize the SQLx connection pool
    let pg_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(options.pg.max_size)
        .connect_with(pg_options)
        .await
        .expect("Failed to connect to PostgreSQL");

    // Initialize the Deadpool-Diesel connection pool
    let manager = Manager::new(&options.pg.url, Runtime::Tokio1);
    let pool = Pool::builder(manager)
        .max_size(options.pg.max_size as usize)
        .build()
        .expect("Failed to initialize Deadpool-Diesel pool");

    // Create the session repository and service
    let session_port: Arc<dyn SessionPort + Send + Sync> = Arc::new(SessionDBRepository::new(pool));
    let session_service = Arc::new(SessionService::new(session_port));

    // Create application state to be shared across workers
    let app_state = AppState::new(session_service);

    // Configure the worker options with the desired concurrency and schema
    let worker = WorkerOptions::default()
        .concurrency(options.worker.concurrent)
        .schema(&options.worker.schema)
        .add_extension(app_state)
        .define_job::<BsKeySharesWorker>() // Register the job for aggregate bootstrapping key shares
        .define_job::<ComputeFunctionWorker>() // Register the job for computing functions
        .pg_pool(pg_pool) // Use the SQLx pool for job storage
        .init()
        .await
        .expect("Failed to initialize worker");

    // Start the worker and process tasks indefinitely
    worker.run().await.expect("Worker encountered an error");
}
