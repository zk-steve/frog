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

/// Represents a session entity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionEntity {
    /// Identifier for the session.
    pub id: SessionId,
    pub status: SessionStatus,
    pub client_info: HashMap<ClientId, ClientEntity>,

    pub pk: Vec<u8>,
    pub encrypted_result: Vec<Vec<u8>>,

    #[serde(skip_serializing, skip_deserializing)]
    pub phantom_server: Option<PhantomServer<NativeOps>>,
}

impl SessionEntity {
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

#[derive(Debug, Serialize, Deserialize, Eq, Hash, PartialEq, Clone)]
pub struct SessionId(pub Uuid);

impl TryFrom<&str> for SessionId {
    type Error = CoreError;

    fn try_from(id: &str) -> Result<Self, Self::Error> {
        Ok(SessionId(
            Uuid::from_str(id).map_err(CoreError::ParseIdError)?,
        ))
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionStatus {
    WaitingForClients,
    WaitingForBootstrap,
    WaitingForArgument,
    Done,
}

impl FromStr for SessionStatus {
    type Err = String;

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionStatus::WaitingForClients => write!(f, "WaitingForClients"),
            SessionStatus::WaitingForBootstrap => write!(f, "WaitingForBootstrap"),
            SessionStatus::WaitingForArgument => write!(f, "WaitingForArgument"),
            SessionStatus::Done => write!(f, "Done"),
        }
    }
}
