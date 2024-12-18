use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use phantom::crs::Crs;
use phantom::native_ops::NativeOps;
use phantom::param::Param;
use phantom::server::PhantomServer;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::client::{ClientEntity, ClientId};
use crate::errors::CoreError;

/// Represents a session entity, which manages the state and data of a session.
///
/// Each `SessionEntity` contains:
/// - A unique session ID (`id`).
/// - The current status of the session (`status`).
/// - Information about the connected clients (`client_info`).
/// - Cryptographic data such as the aggregated public key (`pk`) and encrypted results (`encrypted_result`).
/// - A `PhantomServer` instance for handling Phantom related operations.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionEntity {
    /// Unique identifier for the session.
    pub id: SessionId,
    /// Current status of the session.
    pub status: SessionStatus,
    /// Mapping of client IDs to their respective entities.
    pub client_info: HashMap<ClientId, ClientEntity>,
    /// Public key associated with the session.
    pub pk: Vec<u8>,
    /// Encrypted results generated during the session.
    pub encrypted_result: Vec<Vec<u8>>,

    /// Server-side Phantom related data.
    #[serde(skip_serializing, skip_deserializing)]
    pub phantom_server: Option<PhantomServer<NativeOps>>,
}

impl SessionEntity {
    /// Creates a new `SessionEntity` with the given parameters.
    ///
    /// Initializes the session with default values for `status`, `client_info`,
    /// `encrypted_result`, and `pk`. A `PhantomServer` instance is created using the provided
    /// `phantom_param` and `crs`.
    ///
    /// # Parameters
    /// - `id`: Unique identifier for the session.
    /// - `phantom_param`: Parameters required for the PhantomServer instance.
    /// - `crs`: Common Reference String.
    ///
    /// # Returns
    /// A new `SessionEntity` instance.
    pub fn new(id: SessionId, phantom_param: Param, crs: Crs) -> Self {
        Self {
            id,
            status: SessionStatus::WaitingForClients,
            client_info: Default::default(),
            encrypted_result: Default::default(),
            pk: Default::default(),
            phantom_server: Some(PhantomServer::new(phantom_param, crs, None, None, None).unwrap()),
        }
    }
}

/// Represents a unique identifier for a session, implemented as a wrapper around `Uuid`.
#[derive(Debug, Serialize, Deserialize, Eq, Hash, PartialEq, Clone)]
pub struct SessionId(pub Uuid);

impl TryFrom<&str> for SessionId {
    type Error = CoreError;

    /// Attempts to create a `SessionId` from a string representation of a UUID.
    ///
    /// # Parameters
    /// - `id`: A string containing the UUID.
    ///
    /// # Returns
    /// - `Ok(SessionId)`: If the string is a valid UUID.
    /// - `Err`: If the string cannot be parsed as a UUID.
    fn try_from(id: &str) -> Result<Self, Self::Error> {
        Ok(SessionId(
            Uuid::from_str(id).map_err(CoreError::ParseIdError)?,
        ))
    }
}

impl fmt::Display for SessionId {
    /// Formats the `SessionId` for display.
    ///
    /// Outputs the UUID as a string.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents the status of a session.
///
/// The session can be in one of the following states:
/// - `WaitingForClients`: Waiting for clients to connect.
/// - `WaitingForBootstrap`: Waiting for the bootstrap phase to complete.
/// - `WaitingForArgument`: Waiting for clients to send their arguments.
/// - `Done`: The session is complete.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionStatus {
    WaitingForClients,
    WaitingForBootstrap,
    WaitingForArgument,
    Done,
}

impl FromStr for SessionStatus {
    type Err = String;

    /// Parses a `SessionStatus` from its string representation.
    ///
    /// # Parameters
    /// - `s`: A string representing the session status.
    ///
    /// # Returns
    /// - `Ok(SessionStatus)`: If the string matches a valid status.
    /// - `Err`: If the string does not match any valid status.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "WaitingForClients" => Ok(SessionStatus::WaitingForClients),
            "WaitingForBootstrap" => Ok(SessionStatus::WaitingForBootstrap),
            "WaitingForArgument" => Ok(SessionStatus::WaitingForArgument),
            "Done" => Ok(SessionStatus::Done),
            _ => Err(format!("'{}' is not a valid status", s)),
        }
    }
}

impl fmt::Display for SessionStatus {
    /// Formats the `SessionStatus` for display.
    ///
    /// Outputs the status as a string.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionStatus::WaitingForClients => write!(f, "WaitingForClients"),
            SessionStatus::WaitingForBootstrap => write!(f, "WaitingForBootstrap"),
            SessionStatus::WaitingForArgument => write!(f, "WaitingForArgument"),
            SessionStatus::Done => write!(f, "Done"),
        }
    }
}
