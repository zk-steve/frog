use async_trait::async_trait;
use frog_core::entities::game::GameId;
use frog_core::entities::game_player::{GamePlayerEntity, GamePlayerId};
use frog_core::errors::CoreError;
use frog_core::ports::game_client::GameClientPort;
use reqwest::Client;

pub struct GameClient {
    server_endpoint: String,
    client: Client,
}

impl GameClient {
    pub fn new(server_endpoint: String, client: Client) -> Self {
        Self {
            server_endpoint,
            client,
        }
    }
}

#[async_trait]
impl GameClientPort for GameClient {
    async fn update(
        &self,
        game_id: GameId,
        game_player_id: GamePlayerId,
        game_player_entity: GamePlayerEntity,
    ) -> Result<String, CoreError> {
        let reqwest_response = self
            .client
            .get(&self.server_endpoint)
            .send()
            .await
            .map_err(|e| CoreError::InternalError(e.into()))?;
        Ok("".to_string())
    }
}
