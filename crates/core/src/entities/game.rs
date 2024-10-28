use std::collections::HashMap;
use std::fmt;
use std::io::{Error, ErrorKind};

use serde::{Deserialize, Serialize};

use crate::entities::entity::Entity;
use crate::entities::game_player::GamePlayerEntity;
use crate::entities::player::PlayerId;

/// Represents a question entity.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GameEntity {
    /// Identifier for the question.
    pub id: GameId,

    pub game_data: HashMap<PlayerId, GamePlayerEntity>,
}

impl GameEntity {
    pub fn new(id: GameId) -> Self {
        Self {
            id,
            game_data: Default::default(),
        }
    }
}

impl Entity<GameEntity> for GameEntity {}

#[derive(Debug, Serialize, Deserialize, Eq, Hash, PartialEq, Clone)]
pub struct GameId(pub String);

impl TryFrom<&str> for GameId {
    type Error = Error;

    fn try_from(id: &str) -> Result<Self, Self::Error> {
        match id.is_empty() {
            false => Ok(GameId(id.to_string())),
            true => Err(Error::new(ErrorKind::InvalidInput, "No id provided")),
        }
    }
}

impl fmt::Display for GameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
