use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use frog_core::entities::game::GameId;
use frog_core::entities::game_player::{GamePlayerEntity, GamePlayerId};
use frog_core::errors::CoreError;
use frog_core::ports::game_player::GamePlayerPort;

use crate::in_memory::state::InMemoryState;

pub struct GamePlayerInMemoryRepository {
    inner_state: Arc<RwLock<InMemoryState>>,
}

impl GamePlayerInMemoryRepository {
    pub fn new(state: Arc<RwLock<InMemoryState>>) -> Self {
        Self { inner_state: state }
    }
}

#[async_trait]
impl GamePlayerPort for GamePlayerInMemoryRepository {
    async fn update(
        &self,
        game_id: GameId,
        game_player_id: GamePlayerId,
        game_player_entity: GamePlayerEntity,
    ) -> Result<String, CoreError> {
        todo!()
    }
}
