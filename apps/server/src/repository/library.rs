use colette_core::{
    common::Findable,
    library::{Error, LibraryItem, LibraryItemFindParams, LibraryRepository},
};
use sqlx::{Pool, Postgres};

use super::common;

#[derive(Debug, Clone)]
pub struct PostgresLibraryRepository {
    pool: Pool<Postgres>,
}

impl PostgresLibraryRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresLibraryRepository {
    type Params = LibraryItemFindParams;
    type Output = Result<Vec<LibraryItem>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let mut library_items = common::select_folders(
            &self.pool,
            None,
            params.user_id,
            Some(params.folder_id),
            params.limit,
            None,
        )
        .await
        .map(|e| e.into_iter().map(LibraryItem::Folder).collect::<Vec<_>>())?;

        let mut feeds = common::select_feeds(
            &self.pool,
            None,
            Some(params.folder_id),
            params.user_id,
            None,
            params.limit,
            None,
        )
        .await
        .map(|e| e.into_iter().map(LibraryItem::Feed).collect::<Vec<_>>())?;

        let mut collections = common::select_collections(
            &self.pool,
            None,
            Some(params.folder_id),
            params.user_id,
            params.limit,
            None,
        )
        .await
        .map(|e| {
            e.into_iter()
                .map(LibraryItem::Collection)
                .collect::<Vec<_>>()
        })?;

        library_items.append(&mut feeds);
        library_items.append(&mut collections);

        Ok(library_items)
    }
}

#[async_trait::async_trait]
impl LibraryRepository for PostgresLibraryRepository {}
