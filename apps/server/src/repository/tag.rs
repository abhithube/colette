use colette_core::{
    Tag,
    common::Transaction,
    tag::{
        Error, TagById, TagCreateParams, TagDeleteParams, TagFindByIdsParams, TagFindParams,
        TagRepository, TagUpdateParams,
    },
};
use colette_query::{IntoDelete, IntoInsert, IntoSelect, IntoUpdate};
use sea_orm::{ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr, FromQueryResult};

use super::common::parse_timestamp;

#[derive(Debug, Clone)]
pub struct SqliteTagRepository {
    db: DatabaseConnection,
}

impl SqliteTagRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl TagRepository for SqliteTagRepository {
    async fn find_tags(&self, params: TagFindParams) -> Result<Vec<Tag>, Error> {
        let tags = TagWithCounts::find_by_statement(
            self.db.get_database_backend().build(&params.into_select()),
        )
        .all(&self.db)
        .await
        .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(tags)
    }

    async fn find_tags_by_ids(
        &self,
        tx: &dyn Transaction,
        params: TagFindByIdsParams,
    ) -> Result<Vec<TagById>, Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let tags = tx
            .query_all(self.db.get_database_backend().build(&params.into_select()))
            .await
            .map(|e| {
                e.into_iter()
                    .map(|e| TagById {
                        id: e.try_get_by_index::<String>(0).unwrap().parse().unwrap(),
                        user_id: e.try_get_by_index::<String>(1).unwrap().parse().unwrap(),
                    })
                    .collect()
            })?;

        Ok(tags)
    }

    async fn create_tag(&self, params: TagCreateParams) -> Result<(), Error> {
        let title = params.title.clone();

        self.db
            .execute(self.db.get_database_backend().build(&params.into_insert()))
            .await
            .map_err(|e| match e {
                DbErr::RecordNotInserted => Error::Conflict(title),
                _ => Error::Database(e),
            })?;

        Ok(())
    }

    async fn update_tag(&self, tx: &dyn Transaction, params: TagUpdateParams) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        if params.title.is_none() {
            return Ok(());
        }

        tx.execute(self.db.get_database_backend().build(&params.into_update()))
            .await?;

        Ok(())
    }

    async fn delete_tag(&self, tx: &dyn Transaction, params: TagDeleteParams) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        tx.execute(self.db.get_database_backend().build(&params.into_delete()))
            .await?;

        Ok(())
    }
}

#[derive(sea_orm::FromQueryResult)]
pub struct TagWithCounts {
    pub id: String,
    pub title: String,
    pub user_id: String,
    pub created_at: i32,
    pub updated_at: i32,
    pub feed_count: i64,
    pub bookmark_count: i64,
}

impl From<TagWithCounts> for Tag {
    fn from(value: TagWithCounts) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            title: value.title,
            user_id: value.user_id.parse().unwrap(),
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.updated_at),
            feed_count: Some(value.feed_count),
            bookmark_count: Some(value.bookmark_count),
        }
    }
}
