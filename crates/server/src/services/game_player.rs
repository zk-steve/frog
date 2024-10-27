use std::sync::Arc;

use frog_core::ports::game_player::GamePlayerPort;

pub struct GamePlayerService {
    pub game_player: Arc<dyn GamePlayerPort + Sync + Send>,
}

impl GamePlayerService {
    pub fn new(game_player: Arc<dyn GamePlayerPort + Sync + Send>) -> Self {
        Self { game_player }
    }
}
