use colette_core::{
    bookmark::BookmarkFindParams,
    common::Findable,
    feed::FeedFindParams,
    library::{Error, LibraryItem, LibraryItemFindParams, LibraryRepository},
};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;

use super::{bookmark::BookmarkSelect, feed::FeedSelect, folder::FolderSelect};

#[derive(Debug, Clone)]
pub struct PostgresLibraryRepository {
    pool: Pool,
}

impl PostgresLibraryRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresLibraryRepository {
    type Params = LibraryItemFindParams;
    type Output = Result<Vec<LibraryItem>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) = crate::folder::select(
            None,
            params.user_id,
            Some(params.folder_id),
            params.limit,
            None,
        )
        .build_postgres(PostgresQueryBuilder);

        let stmt = client
            .prepare_cached(&sql)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let mut folders = client
            .query(&stmt, &values.as_params())
            .await
            .map(|e| {
                e.into_iter()
                    .map(|e| LibraryItem::Folder(FolderSelect::from(e).0))
                    .collect::<Vec<_>>()
            })
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) = super::feed::build_select(FeedFindParams {
            folder_id: Some(params.folder_id),
            user_id: params.user_id,
            limit: params.limit,
            ..Default::default()
        })
        .build_postgres(PostgresQueryBuilder);

        let stmt = client
            .prepare_cached(&sql)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let mut feeds = client
            .query(&stmt, &values.as_params())
            .await
            .map(|e| {
                e.into_iter()
                    .map(|e| LibraryItem::Feed(FeedSelect::from(e).0))
                    .collect::<Vec<_>>()
            })
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) = super::bookmark::build_select(BookmarkFindParams {
            folder_id: Some(params.folder_id),
            user_id: params.user_id,
            limit: params.limit,
            ..Default::default()
        })
        .build_postgres(PostgresQueryBuilder);

        let stmt = client
            .prepare_cached(&sql)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let mut bookmarks = client
            .query(&stmt, &values.as_params())
            .await
            .map(|e| {
                e.into_iter()
                    .map(|e| LibraryItem::Bookmark(BookmarkSelect::from(e).0))
                    .collect::<Vec<_>>()
            })
            .map_err(|e| Error::Unknown(e.into()))?;

        let mut library_items = Vec::new();
        library_items.append(&mut folders);
        library_items.append(&mut feeds);
        library_items.append(&mut bookmarks);

        Ok(library_items)
    }
}

impl LibraryRepository for PostgresLibraryRepository {}
