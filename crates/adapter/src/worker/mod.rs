use std::collections::HashMap;
use std::str::FromStr;

use async_trait::async_trait;
use frog_common::workers::{
    Worker, BS_KEY_SHARES_AGGREGATOR_WORKER_IDENTIFIER, COMPUTE_FUNCTION_WORKER_IDENTIFIER,
};
use frog_core::entities::session::SessionId;
use frog_core::errors::CoreError;
use frog_core::errors::CoreError::WorkerError;
use frog_core::ports::worker::WorkerPort;
use graphile_worker::WorkerUtils;
use sqlx::postgres::PgConnectOptions;
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub struct WorkerAdapter {
    pub worker_utils: WorkerUtils,
}

impl WorkerAdapter {
    pub async fn new(url: &str, max_connections: u32, schema: String) -> Self {
        let pg_options = PgConnectOptions::from_str(url).unwrap();
        let pg_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(max_connections)
            .connect_with(pg_options)
            .await
            .unwrap();

        let worker_utils = WorkerUtils::new(pg_pool, schema);
        Self { worker_utils }
    }
}

#[async_trait]
impl WorkerPort for WorkerAdapter {
    async fn aggregate_bs_key_shares(&self, session_id: SessionId) -> Result<(), CoreError> {
        // retrieve the current span
        let span = tracing::Span::current();
        // retrieve the current context
        let cx = span.context();
        // inject the current context through the amqp headers
        let mut tracing_info = HashMap::new();
        opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut tracing_info)
        });
        let payload = Worker {
            data: session_id,
            tracing: tracing_info,
        };
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

    async fn compute_function(&self, session_id: SessionId) -> Result<(), CoreError> {
        // retrieve the current span
        let span = tracing::Span::current();
        // retrieve the current context
        let cx = span.context();
        // inject the current context through the amqp headers
        let mut tracing_info = HashMap::new();
        opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut tracing_info)
        });
        let payload = Worker {
            data: session_id,
            tracing: tracing_info,
        };
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
