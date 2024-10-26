use serde::{Deserialize, Serialize};

pub const DUMMY_WORKER_IDENTIFIER: &str = "dummy";

#[derive(Deserialize, Serialize)]
pub struct DummyWorkerData {
    pub internal_id: String,
}
