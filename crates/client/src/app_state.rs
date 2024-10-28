#[derive(Clone)]
pub struct AppState {}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {}
    }
}
