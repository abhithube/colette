use colette_core::{
    bookmark::BookmarkFindParams,
    common::Findable,
    feed::FeedFindParams,
    library::{Error, LibraryItem, LibraryItemFindParams, LibraryRepository},
};
use deadpool_sqlite::rusqlite;
use deadpool_sqlite::Pool;
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder;

use super::{bookmark::BookmarkSelect, feed::FeedSelect, folder::FolderSelect};

#[derive(Debug, Clone)]
pub struct SqliteLibraryRepository {
    pool: Pool,
}

impl SqliteLibraryRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteLibraryRepository {
    type Params = LibraryItemFindParams;
    type Output = Result<Vec<LibraryItem>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let (sql, values) = crate::folder::select(
                None,
                params.user_id,
                Some(params.folder_id),
                params.limit,
                None,
            )
            .build_rusqlite(SqliteQueryBuilder);

            let mut stmt = conn.prepare_cached(&sql)?;
            let mut rows = stmt.query(&*values.as_params())?;

            let mut library_items: Vec<LibraryItem> = Vec::new();
            while let Some(row) = rows.next()? {
                library_items.push(LibraryItem::Folder(
                    FolderSelect::try_from(row).map(|e| e.0)?,
                ));
            }

            let (sql, values) = super::feed::build_select(FeedFindParams {
                folder_id: Some(params.folder_id),
                user_id: params.user_id,
                limit: params.limit,
                ..Default::default()
            })
            .build_rusqlite(SqliteQueryBuilder);

            let mut stmt = conn.prepare_cached(&sql)?;
            let mut rows = stmt.query(&*values.as_params())?;

            while let Some(row) = rows.next()? {
                library_items.push(LibraryItem::Feed(FeedSelect::try_from(row).map(|e| e.0)?));
            }

            let (sql, values) = super::bookmark::build_select(BookmarkFindParams {
                folder_id: Some(params.folder_id),
                user_id: params.user_id,
                limit: params.limit,
                ..Default::default()
            })
            .build_rusqlite(SqliteQueryBuilder);

            let mut stmt = conn.prepare_cached(&sql)?;
            let mut rows = stmt.query(&*values.as_params())?;

            while let Some(row) = rows.next()? {
                library_items.push(LibraryItem::Bookmark(
                    BookmarkSelect::try_from(row).map(|e| e.0)?,
                ));
            }

            Ok(library_items)
        })
        .await
        .unwrap()
        .map_err(|e: rusqlite::Error| Error::Unknown(e.into()))
    }
}

impl LibraryRepository for SqliteLibraryRepository {}
