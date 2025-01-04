use uuid::Uuid;

use crate::common::{Creatable, Deletable, Findable, IdParams, Updatable};

#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct Collection {
    pub id: Uuid,
    pub title: String,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    pub title: String,
}

pub trait CollectionRepository:
    Findable<Params = CollectionFindParams, Output = Result<Vec<Collection>, Error>>
    + Creatable<Data = CollectionCreateData, Output = Result<Uuid, Error>>
    + Updatable<Params = IdParams, Data = CollectionUpdateData, Output = Result<(), Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
    + 'static
{
}

#[derive(Clone, Debug, Default)]
pub struct CollectionFindParams {
    pub id: Option<Uuid>,
    pub user_id: Uuid,
    pub limit: Option<u64>,
    pub cursor: Option<Cursor>,
}

#[derive(Clone, Debug, Default)]
pub struct CollectionCreateData {
    pub title: String,
    pub user_id: Uuid,
}

#[derive(Clone, Debug, Default)]
pub struct CollectionUpdateData {
    pub title: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("collection not found with ID: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
