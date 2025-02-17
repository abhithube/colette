use uuid::Uuid;

use super::{Collection, Cursor, Error};
use crate::common::{Creatable, Deletable, Findable, IdParams, Updatable};

#[async_trait::async_trait]
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

#[derive(Debug, Clone, Default)]
pub struct CollectionFindParams {
    pub id: Option<Uuid>,
    pub folder_id: Option<Option<Uuid>>,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone, Default)]
pub struct CollectionCreateData {
    pub title: String,
    pub folder_id: Option<Uuid>,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct CollectionUpdateData {
    pub title: Option<String>,
    pub folder_id: Option<Option<Uuid>>,
}
