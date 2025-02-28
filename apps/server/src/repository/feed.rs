use chrono::{DateTime, Utc};
use colette_core::{
    Feed, Tag,
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    feed::{
        ConflictError, Error, FeedCreateData, FeedFindParams, FeedRepository, FeedScrapedData,
        FeedUpdateData,
    },
};
use colette_model::{feeds, user_feed_tags, user_feeds};
use futures::{StreamExt, TryStreamExt, stream::BoxStream};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection, DatabaseTransaction,
    DbErr, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter, QuerySelect, RuntimeErr,
    TransactionTrait, prelude::Expr, sea_query::Func,
};
use sqlx::{
    QueryBuilder,
    types::{Json, Text},
};
use url::Url;
use uuid::{Uuid, fmt::Hyphenated};

use super::common;

#[derive(Debug, Clone)]
pub struct SqliteFeedRepository {
    db: DatabaseConnection,
}

impl SqliteFeedRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteFeedRepository {
    type Params = FeedFindParams;
    type Output = Result<Vec<Feed>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let initial = format!(
            r#"WITH unread_count AS (
  SELECT ufe.user_feed_id, count(ufe.id) AS count
    FROM user_feed_entries ufe
   WHERE ufe.user_id = '{0}' AND NOT ufe.has_read
   GROUP BY ufe.user_feed_id
),
json_tags AS (
  SELECT uft.user_feed_id, json_group_array(json_object('id', t.id, 'title', t.title) ORDER BY t.title) AS tags
    FROM user_feed_tags uft
    JOIN tags t ON t.id = uft.tag_id
   WHERE uft.user_id = '{0}'
   GROUP BY uft.user_feed_id
)
SELECT uf.id,
       f.link,
       uf.title,
       f.xml_url,
       uf.created_at,
       uf.updated_at,
       coalesce(jt.tags, '[]') AS tags,
       coalesce(uc.count, 0) AS unread_count
  FROM user_feeds uf
 INNER JOIN feeds f ON f.id = uf.feed_id
  LEFT JOIN json_tags jt ON jt.user_feed_id = uf.id
  LEFT JOIN unread_count uc ON uc.user_feed_id = uf.id
 WHERE uf.user_id = '{0}'"#,
            Hyphenated::from(params.user_id)
        );

        let mut qb = QueryBuilder::new(initial);

        if let Some(id) = params.id {
            qb.push("\n   AND ufe.id = ");
            qb.push_bind(Hyphenated::from(id));
        }
        if let Some(cursor) = params.cursor {
            let mut separated = qb.separated(", ");
            separated.push_unseparated("\n   AND (uf.title, uf.id) > (");
            separated.push_bind(cursor.title);
            separated.push_bind(cursor.id);
            separated.push_unseparated(")");
        }
        if let Some(tags) = params.tags {
            let mut separated = qb.separated(", ");
            separated.push_unseparated("\n   AND EXISTS (SELECT 1 FROM user_feed_tags uft WHERE uft.user_feed_id = b.id AND uft.tag_id in (");
            for id in tags {
                separated.push_bind(id);
            }
            separated.push(")");
        }

        qb.push("\n ORDER BY uf.title ASC, uf.id ASC");
        if let Some(limit) = params.limit {
            qb.push("\n LIMIT ");
            qb.push_bind(limit);
        }

        let query = qb.build_query_as::<FeedRow>();

        let feeds = query
            .fetch_all(self.db.get_sqlite_connection_pool())
            .await
            .map(|e| e.into_iter().map(Into::into).collect())
            .map_err(|e| DbErr::Query(RuntimeErr::SqlxError(e)))?;

        // let user_id = Hyphenated::from(params.user_id);
        // let id = params.id.map(Hyphenated::from);
        // let skip_id = id.is_none();
        // // let skip_tags = tags.is_none();

        // let mut skip_cursor = true;
        // let mut cursor_title = Option::<String>::None;
        // let mut cursor_id = Option::<Hyphenated>::None;
        // if let Some(cursor) = params.cursor {
        //     skip_cursor = false;
        //     cursor_title = Some(cursor.title);
        //     cursor_id = Some(cursor.id.into());
        // }

        // let feeds = sqlx::query_file_as!(
        //     FeedRow,
        //     "queries/user_feeds/select.sql",
        //     user_id,
        //     skip_id,
        //     params.id,
        //     // skip_tags,
        //     // &params.tags.unwrap_or_default(),
        //     skip_cursor,
        //     cursor_title,
        //     cursor_id,
        //     params.limit
        // )
        // .fetch_all(self.db.get_sqlite_connection_pool())
        // .await
        // .map(|e| e.into_iter().map(Into::into).collect())
        // .map_err(|e| DbErr::Query(RuntimeErr::SqlxError(e)))?;

        Ok(feeds)
    }
}

#[async_trait::async_trait]
impl Creatable for SqliteFeedRepository {
    type Data = FeedCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let tx = self.db.begin().await?;

        let Some(feed) = feeds::Entity::find()
            .filter(
                Condition::any()
                    .add(feeds::Column::Link.eq(data.url.to_string()))
                    .add(feeds::Column::XmlUrl.eq(data.url.to_string())),
            )
            .one(&tx)
            .await?
        else {
            return Err(Error::Conflict(ConflictError::NotCached(data.url)));
        };

        let id = Uuid::new_v4();
        let user_feed = user_feeds::ActiveModel {
            id: ActiveValue::Set(id.into()),
            title: ActiveValue::Set(data.title),
            user_id: ActiveValue::Set(data.user_id.into()),
            feed_id: ActiveValue::Set(feed.id),
            ..Default::default()
        };
        user_feed.insert(&tx).await.map_err(|e| match e {
            DbErr::RecordNotInserted => Error::Conflict(ConflictError::AlreadyExists(data.url)),
            _ => Error::Database(e),
        })?;

        common::insert_many_user_feed_entries(&tx, feed.id).await?;

        if let Some(tags) = data.tags {
            link_tags(&tx, tags, id, data.user_id).await?;
        }

        tx.commit().await?;

        Ok(id)
    }
}

#[async_trait::async_trait]
impl Updatable for SqliteFeedRepository {
    type Params = IdParams;
    type Data = FeedUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let tx = self.db.begin().await?;

        let Some(feed) = user_feeds::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if feed.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        let mut feed = feed.into_active_model();

        if let Some(title) = data.title {
            feed.title = ActiveValue::Set(title);
        }

        if feed.is_changed() {
            feed.update(&tx).await?;
        }

        if let Some(tags) = data.tags {
            link_tags(&tx, tags, params.id, params.user_id).await?;
        }

        tx.commit().await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for SqliteFeedRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let tx = self.db.begin().await?;

        let Some(user_feed) = user_feeds::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if user_feed.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        user_feed.delete(&tx).await?;

        tx.commit().await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl FeedRepository for SqliteFeedRepository {
    async fn save_scraped(&self, data: FeedScrapedData) -> Result<(), Error> {
        if data.link_to_users {
            let tx = self.db.begin().await?;

            let feed_id = common::upsert_feed(&tx, data.feed.link, Some(data.url)).await?;
            common::upsert_entries(&tx, data.feed.entries, feed_id).await?;

            common::insert_many_user_feed_entries(&tx, feed_id).await?;

            tx.commit().await?;
        } else {
            let tx = self.db.begin().await?;

            let feed_id = common::upsert_feed(&tx, data.feed.link, Some(data.url)).await?;
            common::upsert_entries(&tx, data.feed.entries, feed_id).await?;

            tx.commit().await?;
        }

        Ok(())
    }

    async fn stream_urls(&self) -> Result<BoxStream<Result<String, Error>>, Error> {
        let urls = feeds::Entity::find()
            .expr_as(
                Func::coalesce([
                    Expr::col(feeds::Column::XmlUrl).into(),
                    Expr::col(feeds::Column::Link).into(),
                ]),
                "url",
            )
            .inner_join(user_feeds::Entity)
            .into_tuple::<String>()
            .stream(&self.db)
            .await?
            .map_err(Error::Database)
            .boxed();

        Ok(urls)
    }
}

async fn link_tags(
    tx: &DatabaseTransaction,
    tags: Vec<Uuid>,
    user_feed_id: Uuid,
    user_id: Uuid,
) -> Result<(), DbErr> {
    let user_feed_id = user_feed_id.to_string();
    let user_id = user_id.to_string();
    let tag_ids = tags.iter().map(|e| e.to_string());

    user_feed_tags::Entity::delete_many()
        .filter(user_feed_tags::Column::TagId.is_not_in(tag_ids.clone()))
        .exec(tx)
        .await?;

    let models = tag_ids.map(|e| user_feed_tags::ActiveModel {
        user_feed_id: ActiveValue::Set(user_feed_id.clone()),
        tag_id: ActiveValue::Set(e),
        user_id: ActiveValue::Set(user_id.clone()),
        ..Default::default()
    });
    user_feed_tags::Entity::insert_many(models)
        .on_conflict_do_nothing()
        .exec(tx)
        .await?;

    Ok(())
}

#[derive(sqlx::FromRow)]
struct FeedRow {
    id: Hyphenated,
    link: Text<Url>,
    title: String,
    xml_url: Option<Text<Url>>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
    tags: Option<Json<Vec<Tag>>>,
    unread_count: Option<i64>,
}

impl From<FeedRow> for Feed {
    fn from(value: FeedRow) -> Self {
        Self {
            id: value.id.into(),
            link: value.link.0,
            title: value.title,
            xml_url: value.xml_url.map(|e| e.0),
            created_at: value.created_at,
            updated_at: value.updated_at,
            tags: value.tags.map(|e| e.0),
            unread_count: value.unread_count,
        }
    }
}
