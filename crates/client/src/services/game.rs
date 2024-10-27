use std::sync::Arc;

use frog_core::ports::game_client::GameClientPort;

use crate::errors::AppError;

pub struct GameService {
    game_client: Arc<dyn GameClientPort + Sync + Send>,
}

impl GameService {
    pub fn new(game_client: Arc<dyn GameClientPort + Sync + Send>) -> Self {
        Self { game_client }
    }

    pub async fn create(&self) -> Result<(), AppError> {
        Ok(())
    }
}
