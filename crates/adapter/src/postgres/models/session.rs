use std::io::Error;
use std::str::FromStr;
use std::time::SystemTime;

use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
use frog_core::entities::session::{SessionEntity, SessionId, SessionStatus};
use uuid::Uuid;

#[derive(Debug, Queryable, Insertable, Selectable, AsChangeset, Identifiable, Clone)]
#[diesel(table_name = super::super::schema::sessions)]
pub struct SessionModel {
    pub id: Uuid,
    pub status: String,
    pub pk: Vec<u8>,
    pub phantom_server: Vec<u8>,
    pub encrypted_result: Vec<u8>,
    pub client_info: Vec<u8>,

    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

impl TryFrom<SessionEntity> for SessionModel {
    type Error = Error;

    fn try_from(entity: SessionEntity) -> Result<Self, Self::Error> {
        Ok(Self {
            id: entity.id.0,
            status: entity.status.to_string(),
            client_info: bincode::serialize(&entity.client_info).unwrap(),
            encrypted_result: bincode::serialize(&entity.encrypted_result).unwrap(),
            pk: entity.pk,
            phantom_server: bincode::serialize(&entity.phantom_server).unwrap(),

            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        })
    }
}

impl From<SessionModel> for SessionEntity {
    fn from(val: SessionModel) -> Self {
        Self {
            id: SessionId(val.id),
            status: SessionStatus::from_str(&val.status).unwrap(),
            client_info: bincode::deserialize(&val.client_info).unwrap(),
            pk: val.pk,
            encrypted_result: bincode::deserialize(&val.encrypted_result).unwrap(),
            phantom_server: bincode::deserialize(&val.phantom_server).unwrap(),
        }
    }
}
