use std::collections::HashMap;
use std::str::FromStr;

use async_trait::async_trait;
use frog_common::workers::{
    WorkerPayload, BS_KEY_SHARES_AGGREGATOR_WORKER_IDENTIFIER, COMPUTE_FUNCTION_WORKER_IDENTIFIER,
};
use frog_core::entities::session::SessionId;
use frog_core::errors::CoreError;
use frog_core::errors::CoreError::WorkerError;
use frog_core::ports::worker::WorkerPort;
use graphile_worker::WorkerUtils;
use sqlx::postgres::PgConnectOptions;
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Adapter for interacting with the Graphile Worker system.
///
/// This struct provides methods to enqueue tasks in the worker queue using `graphile-worker`.
/// It integrates with OpenTelemetry to propagate tracing information through tasks.
pub struct WorkerAdapter {
    /// Utility for interacting with the worker queue.
    pub worker_utils: WorkerUtils,
}

impl WorkerAdapter {
    /// Creates a new instance of `WorkerAdapter`.
    ///
    /// # Arguments
    /// - `url`: The PostgreSQL connection URL.
    /// - `max_connections`: The maximum number of database connections to maintain.
    /// - `schema`: The database schema for worker jobs.
    ///
    /// # Returns
    /// An instance of `WorkerAdapter`.
    pub async fn new(url: &str, max_connections: u32, schema: String) -> Self {
        // Parse the PostgreSQL connection URL into connection options.
        let pg_options = PgConnectOptions::from_str(url).unwrap();

        // Initialize a connection pool with the specified options and max connections.
        let pg_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(max_connections)
            .connect_with(pg_options)
            .await
            .unwrap();

        // Initialize WorkerUtils with the database pool and schema.
        let worker_utils = WorkerUtils::new(pg_pool, schema);

        Self { worker_utils }
    }
}

#[async_trait]
impl WorkerPort for WorkerAdapter {
    /// Enqueues a task to aggregate BS key shares for the specified session.
    ///
    /// # Arguments
    /// - `session_id`: The unique identifier of the session for which the task is enqueued.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - `Err(CoreError)` if an error occurs while enqueuing the task.
    async fn aggregate_bs_key_shares(&self, session_id: SessionId) -> Result<(), CoreError> {
        // Retrieve the current tracing span and context.
        let span = tracing::Span::current();
        let cx = span.context();

        // Inject the tracing context into a HashMap to propagate it.
        let mut tracing_info = HashMap::new();
        opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut tracing_info)
        });

        // Create the payload containing the session ID and tracing information.
        let payload = WorkerPayload {
            data: session_id,
            tracing: tracing_info,
        };

        // Enqueue the job with the worker system.
        self.worker_utils
            .add_raw_job(
                BS_KEY_SHARES_AGGREGATOR_WORKER_IDENTIFIER,
                payload,
                Default::default(),
            )
            .await
            .map_err(|e| WorkerError(e.into()))?;

        Ok(())
    }

    /// Enqueues a task to compute a function for the specified session.
    ///
    /// # Arguments
    /// - `session_id`: The unique identifier of the session for which the task is enqueued.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - `Err(CoreError)` if an error occurs while enqueuing the task.
    async fn compute_function(&self, session_id: SessionId) -> Result<(), CoreError> {
        // Retrieve the current tracing span and context.
        let span = tracing::Span::current();
        let cx = span.context();

        // Inject the tracing context into a HashMap to propagate it.
        let mut tracing_info = HashMap::new();
        opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut tracing_info)
        });

        // Create the payload containing the session ID and tracing information.
        let payload = WorkerPayload {
            data: session_id,
            tracing: tracing_info,
        };

        // Enqueue the job with the worker system.
        self.worker_utils
            .add_raw_job(
                COMPUTE_FUNCTION_WORKER_IDENTIFIER,
                payload,
                Default::default(),
            )
            .await
            .map_err(|e| WorkerError(e.into()))?;

        Ok(())
    }
}
