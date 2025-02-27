use chrono::{DateTime, Utc};
use colette_core::{
    Tag,
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    tag::{Error, TagCreateData, TagFindParams, TagRepository, TagUpdateData},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel,
    ModelTrait, RuntimeErr, TransactionTrait,
};
use uuid::{Uuid, fmt::Hyphenated};

use super::entity;

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
impl Findable for SqliteTagRepository {
    type Params = TagFindParams;
    type Output = Result<Vec<Tag>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let id = params.id.map(Hyphenated::from);
        let user_id = Hyphenated::from(params.user_id);
        let skip_id = id.is_none();

        let mut skip_cursor = true;
        let mut cursor_title = Option::<String>::None;
        if let Some(cursor) = params.cursor {
            skip_cursor = false;
            cursor_title = Some(cursor.title);
        }

        let tags = sqlx::query_as!(
            Tag,
            r#"WITH feed_count AS (
  SELECT uft.tag_id, coalesce(count(uft.user_feed_id), 0) AS count
    FROM user_feed_tags uft
   WHERE uft.user_id = $1
   GROUP BY uft.tag_id
),
bookmark_count AS (
  SELECT bt.tag_id, coalesce(count(bt.bookmark_id), 0) AS count
    FROM bookmark_tags bt
   WHERE bt.user_id = $1
   GROUP BY bt.tag_id
)
SELECT t.id AS "id: Hyphenated",
       t.title,
       t.created_at AS "created_at: DateTime<Utc>",
       t.updated_at AS "updated_at: DateTime<Utc>",
       fc.count AS "feed_count: i64",
       bc.count AS "bookmark_count: i64"
  FROM tags t
  LEFT JOIN feed_count fc ON fc.tag_id = t.id
  LEFT JOIN bookmark_count bc ON bc.tag_id = t.id
 WHERE t.user_id = $1
   AND ($2 OR t.id = $3)
   AND ($4 OR t.title > $5)
 ORDER BY t.title ASC
 LIMIT coalesce($6, -1)"#,
            user_id,
            skip_id,
            id,
            skip_cursor,
            cursor_title,
            params.limit
        )
        .fetch_all(self.db.get_sqlite_connection_pool())
        .await
        .map_err(|e| DbErr::Query(RuntimeErr::SqlxError(e)))?;

        Ok(tags)
    }
}

#[async_trait::async_trait]
impl Creatable for SqliteTagRepository {
    type Data = TagCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let id = Uuid::new_v4();
        let tag = entity::tags::ActiveModel {
            id: ActiveValue::Set(id.into()),
            title: ActiveValue::Set(data.title.clone()),
            user_id: ActiveValue::Set(data.user_id.into()),
            ..Default::default()
        };
        tag.insert(&self.db).await.map_err(|e| match e {
            DbErr::RecordNotInserted => Error::Conflict(data.title),
            _ => Error::Database(e),
        })?;

        Ok(id)
    }
}

#[async_trait::async_trait]
impl Updatable for SqliteTagRepository {
    type Params = IdParams;
    type Data = TagUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let tx = self.db.begin().await?;

        let Some(tag) = entity::tags::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if tag.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        let mut tag = tag.into_active_model();

        if let Some(title) = data.title {
            tag.title = ActiveValue::Set(title);
        }

        if tag.is_changed() {
            tag.update(&tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for SqliteTagRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let tx = self.db.begin().await?;

        let Some(tag) = entity::api_keys::Entity::find_by_id(params.id)
            .one(&tx)
            .await?
        else {
            return Err(Error::NotFound(params.id));
        };
        if tag.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        tag.delete(&tx).await?;

        tx.commit().await?;

        Ok(())
    }
}

impl TagRepository for SqliteTagRepository {}
