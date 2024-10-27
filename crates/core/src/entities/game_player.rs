use std::fmt;
use std::io::{Error, ErrorKind};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::entities::entity::Entity;

/// Represents a player data in a game.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GamePlayerEntity {
    pub id: GamePlayerId,
}

impl GamePlayerEntity {
    pub fn new(id: GamePlayerId) -> Self {
        Self { id }
    }
}

impl Entity<GamePlayerEntity> for GamePlayerEntity {}

#[derive(Debug, Serialize, Deserialize, Eq, Hash, PartialEq, Clone)]
pub struct GamePlayerId(pub String);

impl FromStr for GamePlayerId {
    type Err = Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match id.is_empty() {
            false => Ok(GamePlayerId(id.to_string())),
            true => Err(Error::new(ErrorKind::InvalidInput, "No id provided")),
        }
    }
}

impl fmt::Display for GamePlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
