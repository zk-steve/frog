use async_trait::async_trait;
use deadpool_diesel::postgres::Pool;
use diesel::{
    delete, insert_into, update, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use frog_core::entities::session::{SessionEntity, SessionId};
use frog_core::errors::CoreError;
use frog_core::ports::session::SessionPort;

use crate::postgres::models::session::SessionModel;
use crate::postgres::schema::sessions::dsl::sessions;
use crate::postgres::schema::sessions::id;

// NOTE: path relative to Cargo.toml
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./src/postgres/migrations");

pub struct SessionDBRepository {
    pub db: Pool,
}

impl SessionDBRepository {
    pub fn new(db: Pool) -> Self {
        SessionDBRepository { db }
    }
}

#[async_trait]
impl SessionPort for SessionDBRepository {
    async fn create(&self, session_entity: SessionEntity) -> Result<SessionId, CoreError> {
        self.db
            .get()
            .await
            .unwrap()
            .interact(move |conn| {
                let session = SessionModel::try_from(session_entity)
                    .map_err(|err| CoreError::InternalError(err.into()))?;
                let response = insert_into(sessions)
                    .values(&session)
                    .get_result::<SessionModel>(conn)
                    .map_err(|err| match err {
                        diesel::result::Error::NotFound => CoreError::NotFound,
                        _ => CoreError::InternalError(err.into()),
                    })?;
                Ok(SessionId(response.id))
            })
            .await
            .unwrap()
    }

    async fn get(&self, session_id: SessionId) -> Result<SessionEntity, CoreError> {
        self.db
            .get()
            .await
            .unwrap()
            .interact(move |conn| {
                let response = sessions
                    .filter(id.eq(session_id.0))
                    .select(SessionModel::as_select())
                    .first(conn)
                    .map_err(|err| match err {
                        diesel::result::Error::NotFound => CoreError::NotFound,
                        _ => CoreError::InternalError(err.into()),
                    })?
                    .into();

                Ok(response)
            })
            .await
            .unwrap()
    }

    async fn update(
        &self,
        session_id: SessionId,
        session_entity: SessionEntity,
    ) -> Result<SessionId, CoreError> {
        assert_eq!(session_id, session_entity.id);
        self.db
            .get()
            .await
            .unwrap()
            .interact(move |conn| {
                let session = SessionModel::try_from(session_entity)
                    .map_err(|err| CoreError::InternalError(err.into()))?;
                let response = update(sessions.filter(id.eq(session.id)))
                    .set(&session)
                    .get_result::<SessionModel>(conn)
                    .map_err(|err| match err {
                        diesel::result::Error::NotFound => CoreError::NotFound,
                        _ => CoreError::InternalError(err.into()),
                    })?;
                Ok(SessionId(response.id))
            })
            .await
            .unwrap()
    }

    async fn delete(&self, session_id: SessionId) -> Result<(), CoreError> {
        self.db
            .get()
            .await
            .unwrap()
            .interact(move |conn| {
                let _ = delete(sessions.filter(id.eq(session_id.0)))
                    .execute(conn)
                    .map_err(|err| match err {
                        diesel::result::Error::NotFound => CoreError::NotFound,
                        _ => CoreError::InternalError(err.into()),
                    })?;

                Ok(())
            })
            .await
            .unwrap()
    }
}
