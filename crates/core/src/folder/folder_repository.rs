use uuid::Uuid;

use super::{Cursor, Error, Folder};
use crate::common::{Creatable, Deletable, Findable, IdParams, NonEmptyString, Updatable};

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

#[derive(Clone, Debug, Default)]
pub struct FolderFindParams {
    pub id: Option<Uuid>,
    pub parent_id: Option<Option<Uuid>>,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Clone, Debug, Default)]
pub struct FolderCreateData {
    pub title: NonEmptyString,
    pub parent_id: Option<Uuid>,
    pub user_id: Uuid,
}

#[derive(Clone, Debug, Default)]
pub struct FolderUpdateData {
    pub title: Option<NonEmptyString>,
    pub parent_id: Option<Option<Uuid>>,
}
