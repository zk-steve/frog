use async_trait::async_trait;

use crate::entities::game::{GameEntity, GameId};
use crate::errors::CoreError;

#[async_trait]
pub trait GamePort {
    async fn create(&self, game_entity: GameEntity) -> Result<String, CoreError>;
    async fn update(&self, game_id: GameId, game_entity: GameEntity) -> Result<String, CoreError>;
    async fn delete(&self, game_id: GameId) -> Result<String, CoreError>;
}
