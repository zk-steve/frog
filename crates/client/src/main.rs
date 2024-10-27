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
use phantom::phantom_zone::{Client, Crs, NativeOps, Param};
use phantom::I_2P_60;
use phantom_zone_evaluator::boolean::fhew::prelude::{DecompositionParam, Modulus};
use tokio::sync::RwLock;
use tokio::time::sleep;
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

/// Frog Client.
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
    if options.client.peer_endpoints.len() != 1 {
        panic!("invalid config for peer endpoints, requires 1");
    }

    let request_client = reqwest::Client::new();
    let session_client: Arc<dyn SessionClientPort + Sync + Send> = Arc::new(SessionClient::new(
        options.client.server_endpoint.clone(),
        request_client.clone(),
    ));

    let peer_client: Arc<dyn PeerPort + Sync + Send> = Arc::new(PeerClient::new(request_client));

    // Parse and pad the crs seed from config
    let mut crs_seed = [0u8; 32]; // Create a zero-filled array of 32 bytes
    let bytes = options.client.crs_seed.as_bytes(); // Get the byte representation of the string
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

    // Parse and pad the client seed from config
    let mut client_seed = [0u8; 32]; // Create a zero-filled array of 32 bytes
    let bytes = options.client.client_seed.as_bytes(); // Get the byte representation of the string
    let len = bytes.len().min(32);
    client_seed[..len].copy_from_slice(&bytes[..len]);

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

    let session_service = Arc::new(SessionService::new(
        options.client.client_id.clone(),
        session_client,
        phantom_client.clone(),
    ));

    let server = tokio::spawn(listen(options.server.clone(), session_service.clone()));

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
    session_service.fetch_result().await.unwrap();

    let mut dec_shares = vec![];
    for endpoint in options.client.peer_endpoints.values() {
        loop {
            let dec_share = peer_client.get_dec_share(endpoint).await.unwrap();
            if !dec_share.is_empty() {
                dec_shares.push(dec_share);
                break;
            }
            sleep(Duration::from_secs(1)).await;
        }
    }
    let result = session_service.dec(dec_shares).await.unwrap();
    info!("result: {:?}", result);

    tokio::try_join!(server).expect("Failed to run server");
}

async fn listen(server: Server, session_service: Arc<SessionService>) {
    let routes = routes(AppState::new(session_service)).layer((
        TraceLayer::new_for_http(),
        // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
        // requests don't hang forever.
        TimeoutLayer::new(Duration::from_secs(10)),
    ));

    let endpoint = format!("{}:{}", server.url.as_str(), server.port);
    let listener = tokio::net::TcpListener::bind(endpoint.clone())
        .await
        .unwrap();
    info!("listening on http://{}", endpoint);
    axum::serve(listener, routes)
        .with_graceful_shutdown(kill_signals::wait_for_kill_signals())
        .await
        .unwrap();
}
