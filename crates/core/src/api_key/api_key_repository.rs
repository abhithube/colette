use uuid::Uuid;

use super::{ApiKey, Cursor, Error};
use crate::common::{Creatable, Deletable, Findable, IdParams, Updatable};

pub trait ApiKeyRepository:
    Findable<Params = ApiKeyFindParams, Output = Result<Vec<ApiKey>, Error>>
    + Creatable<Data = ApiKeyCreateData, Output = Result<Uuid, Error>>
    + Updatable<Params = IdParams, Data = ApiKeyUpdateData, Output = Result<(), Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
    + 'static
{
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyFindParams {
    pub id: Option<Uuid>,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyCreateData {
    pub title: String,
    pub value_hash: String,
    pub value_preview: String,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyUpdateData {
    pub title: Option<String>,
}
