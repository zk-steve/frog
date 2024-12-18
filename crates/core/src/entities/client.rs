use std::fmt;
use std::io::{Error, ErrorKind};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Represents a client in a session.
///
/// Each `ClientEntity` contains:
/// - A unique `ClientId` identifying the client.
/// - Cryptographic key shares (`pk_share`, `rp_key_share`, and `bs_key_share`).
/// - Encrypted data associated with the client.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ClientEntity {
    pub id: ClientId,            // Unique identifier for the client
    pub pk_share: Vec<u8>,       // Public key share for the client
    pub rp_key_share: Vec<u8>,   // Ring packing key share for the client
    pub bs_key_share: Vec<u8>,   // Bootstrap key share for the client
    pub encrypted_data: Vec<u8>, // Encrypted data specific to the client
}

impl ClientEntity {
    /// Constructs a new `ClientEntity` instance with the given ID and key shares.
    ///
    /// The `bs_key_share` and `encrypted_data` fields are initialized to their default empty values.
    ///
    /// # Parameters
    /// - `id`: The unique identifier for the client.
    /// - `pk_share`: The public key share for the client.
    /// - `rp_key_share`: The ring packing key share for the client.
    ///
    /// # Returns
    /// A new `ClientEntity` instance.
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

/// Represents a unique identifier for a client, implemented as a wrapper around `usize`.
#[derive(Debug, Serialize, Deserialize, Eq, Hash, PartialEq, Clone)]
pub struct ClientId(pub usize);

impl FromStr for ClientId {
    type Err = Error;

    /// Parses a `ClientId` from a string.
    ///
    /// # Parameters
    /// - `id`: The string representation of the client ID.
    ///
    /// # Returns
    /// - `Ok(ClientId)`: If the string contains a valid numeric ID.
    /// - `Err`: If the string is empty or the parsing fails.
    fn from_str(id: &str) -> Result<Self, Self::Err> {
        if id.is_empty() {
            Err(Error::new(ErrorKind::InvalidInput, "No id provided"))
        } else {
            match id.parse::<usize>() {
                Ok(parsed_id) => Ok(ClientId(parsed_id)),
                Err(_) => Err(Error::new(ErrorKind::InvalidInput, "Invalid numeric ID")),
            }
        }
    }
}

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
