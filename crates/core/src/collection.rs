use uuid::Uuid;

use crate::common::{Creatable, Deletable, Findable, IdParams, Paginated, Updatable};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Collection {
    pub id: Uuid,
    pub title: String,
    pub folder_id: Option<Uuid>,
    pub bookmark_count: Option<i64>,
}

#[async_trait::async_trait]
pub trait CollectionRepository:
    Findable<Params = IdParams, Output = Result<Collection, Error>>
    + Creatable<Data = CollectionCreateData, Output = Result<Collection, Error>>
    + Updatable<Params = IdParams, Data = CollectionUpdateData, Output = Result<Collection, Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
{
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<String>,
    ) -> Result<Paginated<Collection>, Error>;
}

#[derive(Clone, Debug)]
pub struct CollectionCreateData {
    pub title: String,
    pub folder_id: Option<Uuid>,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct CollectionUpdateData {
    pub title: Option<String>,
    pub folder_id: Option<Option<Uuid>>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("collection not found with ID: {0}")]
    NotFound(Uuid),

    #[error("collection already exists with title: {0}")]
    Conflict(String),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
