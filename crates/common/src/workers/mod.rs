use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Worker<T> {
    pub data: T,
    pub tracing: HashMap<String, String>,
}

pub const BS_KEY_SHARES_AGGREGATOR_WORKER_IDENTIFIER: &str = "bs_key_shares";
pub const COMPUTE_FUNCTION_WORKER_IDENTIFIER: &str = "compute_function";
