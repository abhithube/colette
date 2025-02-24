use uuid::Uuid;

use super::{BookmarkFilter, Collection, Cursor, Error};
use crate::{
    Bookmark, bookmark,
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
};

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
    async fn find_bookmarks(
        &self,
        params: CollectionBookmarkFindParams,
    ) -> Result<Vec<Bookmark>, Error>;
}

#[derive(Debug, Clone, Default)]
pub struct CollectionFindParams {
    pub id: Option<Uuid>,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone)]
pub struct CollectionCreateData {
    pub title: String,
    pub filter: BookmarkFilter,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct CollectionUpdateData {
    pub title: Option<String>,
    pub filter: Option<BookmarkFilter>,
}

#[derive(Debug, Clone)]
pub struct CollectionBookmarkFindParams {
    pub filter: BookmarkFilter,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<bookmark::Cursor>,
}
