use frog_common::options::{default_log, Log};
use serde::Deserialize;

/// Configuration options for the application.
///
/// This struct represents the configuration options for the application, including server settings,
/// database configuration, endpoint for the exporter, service name, and logging configuration.
#[readonly::make]
#[derive(Deserialize, Debug, Clone)]
pub struct Options {
    /// Configuration for the server.
    pub server: Server,
    /// Configuration for the worker.
    pub worker: Worker,
    /// Specifies the configuration of database will be connected.
    pub pg: DBConfig,
    /// The endpoint for the exporter.
    pub exporter_endpoint: String,
    /// The name of the service.
    pub service_name: String,
    /// Configuration for logging, including log level.
    #[serde(default = "default_log")]
    pub log: Log,
}

/// Represents server configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    /// Port number for the server.
    pub port: u16,
    /// URL for the server.
    pub url: String,
}

/// Represents worker configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct Worker {
    /// The number of concurrent jobs that can be run at the same time.
    pub concurrent: usize,
    /// The postgresql schema to use for the worker.
    pub schema: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DBConfig {
    pub url: String,
    pub max_size: u32,
}
