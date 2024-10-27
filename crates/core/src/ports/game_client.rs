use async_trait::async_trait;

use crate::entities::game::GameId;
use crate::entities::game_player::{GamePlayerEntity, GamePlayerId};
use crate::errors::CoreError;

#[async_trait]
pub trait GameClientPort {
    async fn update(
        &self,
        game_id: GameId,
        game_player_id: GamePlayerId,
        game_player_entity: GamePlayerEntity,
    ) -> Result<String, CoreError>;
}
