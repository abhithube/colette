use colette_core::{
    common::Findable,
    library::{Error, LibraryItem, LibraryItemFindParams, LibraryRepository},
};
use sqlx::{Pool, Postgres};

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
        let mut folders = crate::common::select_folders(
            &self.pool,
            None,
            params.user_id,
            Some(params.folder_id),
            params.limit,
            None,
        )
        .await
        .map(|e| e.into_iter().map(LibraryItem::Folder).collect::<Vec<_>>())
        .map_err(|e| Error::Unknown(e.into()))?;

        let mut feeds = crate::common::select_feeds(
            &self.pool,
            None,
            Some(params.folder_id),
            params.user_id,
            None,
            params.limit,
            None,
        )
        .await
        .map(|e| e.into_iter().map(LibraryItem::Feed).collect::<Vec<_>>())
        .map_err(|e| Error::Unknown(e.into()))?;

        let mut bookmarks = crate::common::select_bookmarks(
            &self.pool,
            None,
            Some(params.folder_id),
            params.user_id,
            None,
            params.limit,
            None,
        )
        .await
        .map(|e| e.into_iter().map(LibraryItem::Bookmark).collect::<Vec<_>>())
        .map_err(|e| Error::Unknown(e.into()))?;

        let mut library_items = Vec::new();
        library_items.append(&mut folders);
        library_items.append(&mut feeds);
        library_items.append(&mut bookmarks);

        Ok(library_items)
    }
}

impl LibraryRepository for PostgresLibraryRepository {}
