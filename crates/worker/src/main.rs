mod dummy_worker;
mod options;
mod routes;

use std::str::FromStr;
use std::time::Duration;

use clap::{Parser, Subcommand};
use common::cli_args::CliArgs;
use common::kill_signals;
use common::loggers::telemetry::init_telemetry;
use graphile_worker::WorkerOptions;
use opentelemetry::global;
use sqlx::postgres::PgConnectOptions;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::dummy_worker::DummyWorker;
use crate::options::Options;
use crate::routes::routes;

#[tokio::main]
async fn main() {
    let options: Options = CliArgs::default_run_or_get_options(env!("APP_VERSION"));

    init_telemetry(
        options.service_name.as_str(),
        options.exporter_endpoint.as_str(),
        options.log.level.as_str(),
    );

    let server = tokio::spawn(serve(options.clone()));

    run_workers(options).await;

    // Wait for the server to finish shutting down
    tokio::try_join!(server).expect("Failed to run server");

    global::shutdown_tracer_provider();
    info!("Shutdown successfully!");
}

/// Frog Worker.
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
    let routes = routes().layer((
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

pub async fn run_workers(options: Options) {
    let pg_options = PgConnectOptions::from_str(&options.pg.url).unwrap();

    let pg_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(options.pg.max_size)
        .connect_with(pg_options)
        .await
        .unwrap();

    let worker = WorkerOptions::default()
        .concurrency(options.worker.concurrent)
        .schema(options.worker.schema.as_str())
        .define_job::<DummyWorker>()
        .pg_pool(pg_pool)
        .init()
        .await
        .unwrap();

    worker.run().await.unwrap();
    info!("worker is running");
}
