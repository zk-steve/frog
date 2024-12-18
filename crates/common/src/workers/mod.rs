use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Represents a payload for a worker entity that handles a specific task.
///
/// The `WorkerPayload` struct is generic, allowing workers to handle different types of data.
///
/// # Generic Parameters
/// - `T`: The type of the data associated with the worker.
#[derive(Deserialize, Serialize, Debug)]
pub struct WorkerPayload<T> {
    /// The data that the worker processes.
    ///
    /// This could be any type that is required for the task the worker performs.
    pub data: T,

    /// A collection of tracing information for debugging or logging purposes.
    ///
    /// This field stores key-value pairs that can help trace the worker's actions across
    /// different services or components in a system.
    pub tracing: HashMap<String, String>,
}

/// Identifier for the worker responsible for aggregating bootstrap key shares.
///
/// Workers with this identifier are tasked with aggregating bootstrap key shares
/// from various clients or participants in the session.
pub const BS_KEY_SHARES_AGGREGATOR_WORKER_IDENTIFIER: &str = "bs_key_shares";

/// Identifier for the worker responsible for performing a specific computation.
///
/// Workers with this identifier are responsible for executing cryptographic calculations
/// as part of a session.
pub const COMPUTE_FUNCTION_WORKER_IDENTIFIER: &str = "compute_function";
