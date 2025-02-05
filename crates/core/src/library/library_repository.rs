use uuid::Uuid;

use super::{Error, LibraryItem};
use crate::common::Findable;

#[async_trait::async_trait]
pub trait LibraryRepository:
    Findable<Params = LibraryItemFindParams, Output = Result<Vec<LibraryItem>, Error>>
    + Send
    + Sync
    + 'static
{
}

#[derive(Clone, Debug, Default)]
pub struct LibraryItemFindParams {
    pub folder_id: Option<Uuid>,
    pub user_id: Uuid,
    pub limit: Option<i64>,
}
