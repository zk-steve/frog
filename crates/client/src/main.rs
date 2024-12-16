use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use clap::{Parser, Subcommand};
use frog_adapter::http::peer::PeerClient;
use frog_adapter::http::session::SessionClient;
use frog_client::app_state::AppState;
use frog_client::options::{Options, Server};
use frog_client::routes::routes;
use frog_client::services::session::SessionService;
use frog_common::cli_args::CliArgs;
use frog_common::kill_signals;
use frog_common::loggers::telemetry::init_telemetry;
use frog_core::entities::session::SessionStatus;
use frog_core::ports::peer::PeerPort;
use frog_core::ports::session_client::SessionClientPort;
use opentelemetry::global;
use phantom::client::Client;
use phantom::crs::Crs;
use phantom::native_ops::NativeOps;
use phantom::param::Param;
use phantom::utils::{pad_seed_to_32_bytes, I_2P_60};
use phantom_zone_evaluator::boolean::fhew::prelude::{DecompositionParam, Modulus};
use tokio::sync::RwLock;
use tokio::time::sleep;
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

    // Start the server and application flow.
    let server_task = tokio::spawn(serve(options));

    // Wait for the server and other tasks to finish gracefully.
    tokio::try_join!(server_task).expect("Failed to run server");

    // Shutdown the tracing provider before exiting.
    global::shutdown_tracer_provider();
    info!("Shutdown successfully!");
}

/// Command-line arguments for the Frog Client application.
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

/// Subcommands supported by the Frog Client application.
#[derive(Subcommand, Clone, Debug)]
enum Commands {
    /// Print the current configuration.
    Config,
}

/// Starts the server and initializes application services.
///
/// # Arguments
/// - `options`: Configuration options loaded from CLI or configuration files.
pub async fn serve(options: Options) {
    if options.client.peer_endpoints.len() != 1 {
        panic!("Invalid configuration: exactly one peer endpoint is required.");
    }

    // Initialize HTTP clients for interacting with other services.
    let request_client = reqwest::Client::new();
    let session_client: Arc<dyn SessionClientPort + Sync + Send> = Arc::new(SessionClient::new(
        options.client.server_endpoint.clone(),
        request_client.clone(),
    ));
    let peer_client: Arc<dyn PeerPort + Sync + Send> = Arc::new(PeerClient::new(request_client));

    // Parse CRS seed from configuration and ensure it's 32 bytes long.
    let crs_seed = pad_seed_to_32_bytes(options.client.crs_seed.as_bytes());

    // Initialize the CRS and Phantom client parameters.
    let crs = Crs::new(crs_seed);
    let phantom_param = create_phantom_param();

    // Parse client seed from configuration and ensure it's 32 bytes long.
    let client_seed = pad_seed_to_32_bytes(options.client.client_seed.as_bytes());

    // Create the Phantom client with the provided parameters.
    let phantom_client = Arc::new(RwLock::new(
        Client::<NativeOps>::new(
            phantom_param,
            crs,
            options.client.client_id.0,
            client_seed,
            None,
        )
        .unwrap(),
    ));

    // Initialize the session service.
    let session_service = Arc::new(SessionService::new(
        options.client.client_id.clone(),
        options.client.session_id.clone(),
        session_client,
        phantom_client.clone(),
    ));

    // Start the server and main application flow.
    let server_task = tokio::spawn(listen(options.server.clone(), session_service.clone()));
    let main_flow_task = tokio::spawn(main_flow(
        session_service.clone(),
        peer_client.clone(),
        options.client.peer_endpoints.clone(),
    ));
    tokio::try_join!(main_flow_task, server_task).expect("Failed to run server");
}

/// Starts the HTTP server and listens for incoming requests.
///
/// # Arguments
/// - `server`: Server configuration.
/// - `session_service`: Shared session service instance.
async fn listen(server: Server, session_service: Arc<SessionService>) {
    // Define the application's routes with tracing and request timeout layers.
    let routes = routes(AppState::new(session_service)).layer((
        TraceLayer::new_for_http(),
        TimeoutLayer::new(Duration::from_secs(10)), // Ensure requests don't hang indefinitely.
    ));

    // Bind the server to the specified endpoint.
    let endpoint = format!("{}:{}", server.url.as_str(), server.port);
    let listener = tokio::net::TcpListener::bind(&endpoint)
        .await
        .unwrap_or_else(|e| {
            panic!("Failed to bind server to {}: {}", endpoint, e);
        });

    info!("Listening on http://{}", endpoint);

    // Serve HTTP requests with graceful shutdown support.
    axum::serve(listener, routes)
        .with_graceful_shutdown(kill_signals::wait_for_kill_signals())
        .await
        .unwrap();
}

/// Main application flow.
async fn main_flow(
    session_service: Arc<SessionService>,
    peer_client: Arc<dyn PeerPort + Sync + Send>,
    peer_endpoints: HashMap<String, String>,
) {
    session_service.join().await.unwrap();
    session_service
        .wait(SessionStatus::WaitingForBootstrap)
        .await
        .unwrap();
    session_service.update_pk().await.unwrap();
    session_service.bootstrap().await.unwrap();
    session_service
        .wait(SessionStatus::WaitingForArgument)
        .await
        .unwrap();
    session_service.send_secret_data().await.unwrap();
    session_service.wait(SessionStatus::Done).await.unwrap();
    session_service.fetch_encrypted_result().await.unwrap();

    // Collect decryption shares from all peers.
    let mut dec_shares = vec![];
    for endpoint in peer_endpoints.values() {
        loop {
            if let Ok(dec_share) = peer_client.get_dec_share(endpoint).await {
                if !dec_share.is_empty() {
                    dec_shares.push(dec_share);
                    break;
                }
            }
            sleep(Duration::from_secs(1)).await; // Retry until successful.
        }
    }

    // Decrypt and log the final result.
    let result = session_service.decrypt_result(dec_shares).await.unwrap();
    info!("RESULT: {}", result);
}

/// Creates Phantom client parameters.
///
/// # Returns
/// A configured `Param` instance.
fn create_phantom_param() -> Param {
    Param {
        param: I_2P_60,
        ring_packing_modulus: Some(Modulus::Prime(2305843009213554689)),
        ring_packing_auto_decomposition_param: DecompositionParam {
            log_base: 20,
            level: 1,
        },
    }
}
