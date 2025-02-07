use uuid::Uuid;

use super::{Cursor, Error, Folder};
use crate::common::{Creatable, Deletable, Findable, IdParams, Updatable};

#[async_trait::async_trait]
pub trait FolderRepository:
    Findable<Params = FolderFindParams, Output = Result<Vec<Folder>, Error>>
    + Creatable<Data = FolderCreateData, Output = Result<Uuid, Error>>
    + Updatable<Params = IdParams, Data = FolderUpdateData, Output = Result<(), Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
    + 'static
{
}

#[derive(Debug, Clone, Default)]
pub struct FolderFindParams {
    pub id: Option<Uuid>,
    pub parent_id: Option<Option<Uuid>>,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone, Default)]
pub struct FolderCreateData {
    pub title: String,
    pub parent_id: Option<Uuid>,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct FolderUpdateData {
    pub title: Option<String>,
    pub parent_id: Option<Option<Uuid>>,
}
