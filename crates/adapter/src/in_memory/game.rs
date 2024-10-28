use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use frog_core::entities::game::{GameEntity, GameId};
use frog_core::errors::CoreError;
use frog_core::ports::game::GamePort;

use crate::in_memory::state::InMemoryState;

pub struct GameInMemoryRepository {
    inner_state: Arc<RwLock<InMemoryState>>,
}

impl GameInMemoryRepository {
    pub fn new(state: Arc<RwLock<InMemoryState>>) -> Self {
        Self { inner_state: state }
    }
}

#[async_trait]
impl GamePort for GameInMemoryRepository {
    async fn create(&self, game_entity: GameEntity) -> Result<String, CoreError> {
        todo!()
    }

    async fn update(&self, game_id: GameId, game_entity: GameEntity) -> Result<String, CoreError> {
        todo!()
    }

    async fn delete(&self, game_id: GameId) -> Result<String, CoreError> {
        todo!()
    }
}
