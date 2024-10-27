use std::sync::Arc;

use graphile_worker::WorkerUtils;

use crate::services::game::GameService;
use crate::services::game_player::GamePlayerService;

#[derive(Clone)]
pub struct AppState {
    pub worker_utils: Arc<WorkerUtils>,
    pub game_service: Arc<GameService>,
    pub game_player_service: Arc<GamePlayerService>,
}

impl AppState {
    pub fn new(
        worker_utils: Arc<WorkerUtils>,
        game_service: Arc<GameService>,
        game_player_service: Arc<GamePlayerService>,
    ) -> Self {
        Self {
            worker_utils,
            game_service,
            game_player_service,
        }
    }
}
