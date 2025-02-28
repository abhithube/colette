use colette_core::{
    Bookmark, Collection,
    collection::{
        CollectionBookmarkFindParams, CollectionCreateData, CollectionFindParams,
        CollectionRepository, CollectionUpdateData, Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
};
use colette_model::collections;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    IntoActiveModel, ModelTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait, RuntimeErr,
    TransactionTrait,
};
use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::repository::{bookmark::BookmarkRow, common::ToSql};

#[derive(Debug, Clone)]
pub struct SqliteCollectionRepository {
    db: DatabaseConnection,
}

impl SqliteCollectionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteCollectionRepository {
    type Params = CollectionFindParams;
    type Output = Result<Vec<Collection>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let collections = collections::Entity::find()
            .filter(collections::Column::UserId.eq(params.user_id.to_string()))
            .apply_if(params.id, |query, id| {
                query.filter(collections::Column::Id.eq(id.to_string()))
            })
            .apply_if(params.cursor, |query, cursor| {
                query.filter(collections::Column::Title.gt(cursor.title))
            })
            .order_by_asc(collections::Column::Title)
            .limit(params.limit.map(|e| e as u64))
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(collections)
    }
}

#[async_trait::async_trait]
impl Creatable for SqliteCollectionRepository {
    type Data = CollectionCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let id = Uuid::new_v4();
        let collection = collections::ActiveModel {
            id: ActiveValue::Set(id.into()),
            title: ActiveValue::Set(data.title.clone()),
            filter_raw: ActiveValue::Set(serde_json::to_string(&data.filter).unwrap()),
            user_id: ActiveValue::Set(data.user_id.into()),
            ..Default::default()
        };
        collection.insert(&self.db).await.map_err(|e| match e {
            DbErr::RecordNotInserted => Error::Conflict(data.title),
            _ => Error::Database(e),
        })?;

        Ok(id)
    }
}

#[async_trait::async_trait]
impl Updatable for SqliteCollectionRepository {
    type Params = IdParams;
    type Data = CollectionUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let tx = self.db.begin().await?;

        let Some(collection) = collections::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if collection.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        let mut collection = collection.into_active_model();

        if let Some(title) = data.title {
            collection.title = ActiveValue::Set(title);
        }
        if let Some(filter) = data.filter {
            collection.filter_raw = ActiveValue::Set(serde_json::to_string(&filter).unwrap());
        }

        if collection.is_changed() {
            collection.update(&tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for SqliteCollectionRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let tx = self.db.begin().await?;

        let Some(collection) = collections::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if collection.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        collection.delete(&tx).await?;

        tx.commit().await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl CollectionRepository for SqliteCollectionRepository {
    async fn find_bookmarks(
        &self,
        params: CollectionBookmarkFindParams,
    ) -> Result<Vec<Bookmark>, Error> {
        let initial = format!(
            r#"WITH json_tags AS (
  SELECT bt.bookmark_id,
         json_group_array(json_object('id', t.id, 'title', t.title) ORDER BY t.title) AS tags
    FROM bookmark_tags bt
    JOIN tags t ON t.id = bt.tag_id
   WHERE bt.user_id = '{0}'
   GROUP BY bt.bookmark_id
)
SELECT b.id,
       b.link,
       b.title,
       b.thumbnail_url,
       b.published_at,
       b.author,
       b.archived_path,
       b.created_at,
       b.updated_at,
       coalesce(jt.tags, '[]'::jsonb) AS tags
  FROM bookmarks b
  LEFT JOIN json_tags jt ON jt.bookmark_id = b.id
 WHERE b.user_id = '{0}'"#,
            params.user_id
        );

        let mut qb = QueryBuilder::new(initial);

        let where_clause = params.filter.to_sql();
        if !where_clause.is_empty() {
            qb.push(" AND ");
            qb.push(&where_clause);
        }

        if let Some(cursor) = params.cursor {
            qb.push(" AND b.created_at > ");
            qb.push_bind(cursor.created_at);
        }

        qb.push("\n ORDER BY b.created_at ASC");

        if let Some(limit) = params.limit {
            qb.push("\n LIMIT ");
            qb.push_bind(limit);
        }

        let query = qb.build_query_as::<BookmarkRow>();

        let bookmarks = query
            .fetch_all(self.db.get_sqlite_connection_pool())
            .await
            .map(|e| e.into_iter().map(Into::into).collect())
            .map_err(|e| DbErr::Query(RuntimeErr::SqlxError(e)))?;

        Ok(bookmarks)
    }
}
