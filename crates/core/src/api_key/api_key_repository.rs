use uuid::Uuid;

use super::{ApiKey, ApiKeySearched, Cursor, Error};
use crate::common::{Creatable, Deletable, Findable, IdParams, Updatable};

#[async_trait::async_trait]
pub trait ApiKeyRepository:
    Findable<Params = ApiKeyFindParams, Output = Result<Vec<ApiKey>, Error>>
    + Creatable<Data = ApiKeyCreateData, Output = Result<Uuid, Error>>
    + Updatable<Params = IdParams, Data = ApiKeyUpdateData, Output = Result<(), Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
    + 'static
{
    async fn search(&self, params: ApiKeySearchParams) -> Result<Option<ApiKeySearched>, Error>;
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
    pub lookup_hash: String,
    pub verification_hash: String,
    pub title: String,
    pub preview: String,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyUpdateData {
    pub title: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeySearchParams {
    pub lookup_hash: String,
}
