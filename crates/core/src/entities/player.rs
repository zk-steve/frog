use std::fmt;
use std::io::{Error, ErrorKind};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::entities::entity::Entity;

/// Represents a player data in a game.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PlayerEntity {
    pub id: PlayerId,
}

impl PlayerEntity {
    pub fn new(id: PlayerId) -> Self {
        Self { id }
    }
}

impl Entity<PlayerEntity> for PlayerEntity {}

#[derive(Debug, Serialize, Deserialize, Eq, Hash, PartialEq, Clone)]
pub struct PlayerId(pub String);

impl FromStr for PlayerId {
    type Err = Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match id.is_empty() {
            false => Ok(PlayerId(id.to_string())),
            true => Err(Error::new(ErrorKind::InvalidInput, "No id provided")),
        }
    }
}

impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
