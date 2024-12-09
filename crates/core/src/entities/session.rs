use std::collections::HashMap;
use std::fmt;
use std::io::{Error, ErrorKind};

use phantom::phantom_zone::{Crs, NativeOps, Param, PhantomServer};
use serde::{Deserialize, Serialize};

use crate::entities::client::{ClientEntity, ClientId};

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
pub struct SessionId(pub String);

impl TryFrom<&str> for SessionId {
    type Error = Error;

    fn try_from(id: &str) -> Result<Self, Self::Error> {
        match id.is_empty() {
            false => Ok(SessionId(id.to_string())),
            true => Err(Error::new(ErrorKind::InvalidInput, "No id provided")),
        }
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
