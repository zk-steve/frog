use std::fmt;
use std::io::{Error, ErrorKind};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Represents a client in a session.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ClientEntity {
    pub id: ClientId,
    pub pk_share: Vec<u8>,
    pub rp_key_share: Vec<u8>,
    pub bs_key_share: Vec<u8>,
    pub encrypted_data: Vec<u8>,
}

impl ClientEntity {
    pub fn new(id: ClientId, pk_share: Vec<u8>, rp_key_share: Vec<u8>) -> Self {
        Self {
            id,
            pk_share,
            rp_key_share,
            bs_key_share: Default::default(),
            encrypted_data: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, Hash, PartialEq, Clone)]
pub struct ClientId(pub usize);

impl FromStr for ClientId {
    type Err = Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match id.is_empty() {
            false => Ok(ClientId(id.to_string().parse().unwrap())),
            true => Err(Error::new(ErrorKind::InvalidInput, "No id provided")),
        }
    }
}

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
