use std::collections::HashMap;

use frog_common::options::{default_log, Log};
use frog_core::entities::player::PlayerId;
use serde::Deserialize;

/// Configuration options for the application.
///
/// This struct represents the configuration options for the application, including server settings,
/// database configuration, endpoint for the exporter, service name, and logging configuration.
#[readonly::make]
#[derive(Deserialize, Debug)]
pub struct Options {
    /// Configuration for the server.
    pub server: Server,
    /// The endpoint for the exporter.
    pub exporter_endpoint: String,
    /// The name of the service.
    pub service_name: String,
    /// Configuration for the game.
    pub game: Game,
    /// Configuration for logging, including log level.
    #[serde(default = "default_log")]
    pub log: Log,
}

/// Represents server configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct Game {
    /// The endpoint of the game server.
    pub server_endpoint: String,
    /// Player ID
    pub player_id: PlayerId,
    /// The endpoints of other players (player, end_point).
    pub player_endpoints: HashMap<String, String>,
}

/// Represents server configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    /// Port number for the server.
    pub port: u16,
    /// URL for the server.
    pub url: String,
}
