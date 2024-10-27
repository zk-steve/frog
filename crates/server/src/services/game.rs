use std::sync::Arc;

use frog_core::entities::game::GameEntity;
use frog_core::ports::game::GamePort;

use crate::errors::AppError;

pub struct GameService {
    game: Arc<dyn GamePort + Sync + Send>,
}

impl GameService {
    pub fn new(game: Arc<dyn GamePort + Sync + Send>) -> Self {
        Self { game }
    }

    pub async fn create(&self) -> Result<(), AppError> {
        // Right now, this acts as a reset function
        self.game
            .create(GameEntity::new("aas".try_into().unwrap()))
            .await?;
        Ok(())
    }
}
