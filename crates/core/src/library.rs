use uuid::Uuid;

use crate::{
    common::{Findable, Paginated},
    Bookmark, Feed, Folder,
};

#[derive(Clone, Debug)]
pub enum LibraryItem {
    Folder(Folder),
    Feed(Feed),
    Bookmark(Bookmark),
}

#[derive(Clone, Debug, Default)]
pub struct LibraryItemListQuery {
    pub folder_id: Option<Uuid>,
    pub cursor: Option<String>,
}

pub struct LibraryService {
    repository: Box<dyn LibraryRepository>,
}

impl LibraryService {
    pub fn new(repository: impl LibraryRepository) -> Self {
        Self {
            repository: Box::new(repository),
        }
    }

    pub async fn list_library_items(
        &self,
        query: LibraryItemListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<LibraryItem>, Error> {
        let library_items = self
            .repository
            .find(LibraryItemFindParams {
                folder_id: query.folder_id,
                user_id,
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: library_items,
            cursor: None,
        })
    }
}

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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
