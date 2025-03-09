use std::collections::HashMap;

use colette_core::{
    Bookmark,
    bookmark::{
        BookmarkById, BookmarkCreateParams, BookmarkDeleteParams, BookmarkFindByIdParams,
        BookmarkFindParams, BookmarkRepository, BookmarkTagsLinkParams, BookmarkUpdateParams,
        BookmarkUpsertParams, Error,
    },
    common::Transaction,
};
use colette_model::{BookmarkRow, BookmarkRowWithTagRows, BookmarkTagRow};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect, IntoUpdate,
    bookmark_tag::{BookmarkTagDeleteMany, BookmarkTagSelectMany, BookmarkTagUpsertMany},
};
use sea_orm::{ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr, FromQueryResult};

#[derive(Debug, Clone)]
pub struct SqliteBookmarkRepository {
    db: DatabaseConnection,
}

impl SqliteBookmarkRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl BookmarkRepository for SqliteBookmarkRepository {
    async fn find_bookmarks(&self, params: BookmarkFindParams) -> Result<Vec<Bookmark>, Error> {
        let bookmark_rows = BookmarkRow::find_by_statement(
            self.db.get_database_backend().build(&params.into_select()),
        )
        .all(&self.db)
        .await?;

        let select_many = BookmarkTagSelectMany {
            bookmark_ids: bookmark_rows.iter().map(|e| e.id.as_str()),
        };

        let tag_rows = BookmarkTagRow::find_by_statement(
            self.db
                .get_database_backend()
                .build(&select_many.into_select()),
        )
        .all(&self.db)
        .await?;

        let mut tag_row_map = HashMap::<String, Vec<BookmarkTagRow>>::new();

        for row in tag_rows {
            tag_row_map
                .entry(row.bookmark_id.clone())
                .or_default()
                .push(row);
        }

        let bookmarks = bookmark_rows
            .into_iter()
            .map(|bookmark| {
                BookmarkRowWithTagRows {
                    tags: tag_row_map.remove(&bookmark.id),
                    bookmark,
                }
                .into()
            })
            .collect();

        Ok(bookmarks)
    }

    async fn find_bookmark_by_id(
        &self,
        tx: &dyn Transaction,
        params: BookmarkFindByIdParams,
    ) -> Result<BookmarkById, Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let id = params.id;

        let Some(result) = tx
            .query_one(self.db.get_database_backend().build(&params.into_select()))
            .await?
        else {
            return Err(Error::NotFound(id));
        };

        Ok(BookmarkById {
            id: result
                .try_get_by_index::<String>(0)
                .unwrap()
                .parse()
                .unwrap(),
            user_id: result
                .try_get_by_index::<String>(1)
                .unwrap()
                .parse()
                .unwrap(),
        })
    }

    async fn create_bookmark(
        &self,
        tx: &dyn Transaction,
        params: BookmarkCreateParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let url = params.url.clone();

        tx.execute(self.db.get_database_backend().build(&params.into_insert()))
            .await
            .map_err(|e| match e {
                DbErr::RecordNotInserted => Error::Conflict(url),
                _ => Error::Database(e),
            })?;

        Ok(())
    }

    async fn update_bookmark(
        &self,
        tx: Option<&dyn Transaction>,
        params: BookmarkUpdateParams,
    ) -> Result<(), Error> {
        let tx = tx.map(|e| e.as_any().downcast_ref::<DatabaseTransaction>().unwrap());

        if params.title.is_none()
            && params.thumbnail_url.is_none()
            && params.published_at.is_none()
            && params.author.is_none()
            && params.archived_path.is_none()
        {
            return Ok(());
        }

        let statement = self.db.get_database_backend().build(&params.into_update());

        if let Some(tx) = tx {
            tx.execute(statement).await?;
        } else {
            self.db.execute(statement).await?;
        }

        Ok(())
    }

    async fn delete_bookmark(
        &self,
        tx: &dyn Transaction,
        params: BookmarkDeleteParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        tx.execute(self.db.get_database_backend().build(&params.into_delete()))
            .await?;

        Ok(())
    }

    async fn upsert(&self, params: BookmarkUpsertParams) -> Result<(), Error> {
        self.db
            .execute(self.db.get_database_backend().build(&params.into_insert()))
            .await?;

        Ok(())
    }

    async fn link_tags(
        &self,
        tx: &dyn Transaction,
        params: BookmarkTagsLinkParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let delete_many = BookmarkTagDeleteMany {
            bookmark_id: params.bookmark_id,
            tag_ids: params.tags.iter().map(|e| e.id.to_string()),
        };

        tx.execute(
            self.db
                .get_database_backend()
                .build(&delete_many.into_delete()),
        )
        .await?;

        let upsert_many = BookmarkTagUpsertMany {
            bookmark_id: params.bookmark_id,
            tags: params.tags,
        };

        tx.execute(
            self.db
                .get_database_backend()
                .build(&upsert_many.into_insert()),
        )
        .await?;

        Ok(())
    }
}
