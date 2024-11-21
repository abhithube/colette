use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, TagsLinkAction, TagsLinkData, Updatable},
    feed::{
        Error, FeedCacheData, FeedCreateData, FeedFindParams, FeedRepository, FeedUpdateData,
        ProcessedFeed,
    },
    Feed,
};
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use sea_query::{Expr, ExprTrait, SqliteQueryBuilder};
use sea_query_binder::SqlxBinder;
use sqlx::{types::Json, SqliteConnection, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteFeedRepository {
    pool: SqlitePool,
}

impl SqliteFeedRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteFeedRepository {
    type Params = FeedFindParams;
    type Output = Result<Vec<Feed>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let jsonb_agg = Expr::cust(
            r#"JSON_GROUP_ARRAY(JSON_OBJECT('id', HEX("tags"."id"), 'title', "tags"."title") ORDER BY "tags"."title") FILTER (WHERE "tags"."id" IS NOT NULL)"#,
        );

        let tags_subquery = params.tags.map(|e| {
            Expr::cust_with_expr(
                r#"EXISTS (SELECT 1 FROM JSON_EACH("json_tags"."tags") AS "t" WHERE ?)"#,
                Expr::cust(r#""t"."value" ->> 'title'"#).is_in(e),
            )
        });

        let (sql, values) = colette_sql::profile_feed::select(
            params.id,
            params.profile_id,
            params.pinned,
            params.cursor,
            params.limit,
            jsonb_agg,
            tags_subquery,
        )
        .build_sqlx(SqliteQueryBuilder);

        sqlx::query_as_with::<_, FeedSelect, _>(&sql, values)
            .fetch_all(&self.pool)
            .await
            .map(|e| e.into_iter().map(Feed::from).collect::<Vec<_>>())
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for SqliteFeedRepository {
    type Data = FeedCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let feed_id = {
            let (sql, values) =
                colette_sql::feed::select_by_url(data.url.clone()).build_sqlx(SqliteQueryBuilder);

            sqlx::query_scalar_with::<_, i32, _>(&sql, values)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::Conflict(data.url),
                    _ => Error::Unknown(e.into()),
                })?
        };

        let pf_id = {
            let (mut sql, mut values) =
                colette_sql::profile_feed::select_by_unique_index(data.profile_id, feed_id)
                    .build_sqlx(SqliteQueryBuilder);

            if let Some(id) = sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?
            {
                id
            } else {
                (sql, values) = colette_sql::profile_feed::insert(
                    Some(Uuid::new_v4()),
                    None,
                    feed_id,
                    data.profile_id,
                )
                .build_sqlx(SqliteQueryBuilder);

                sqlx::query_scalar_with(&sql, values)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
            }
        };

        if let Some(tags) = data.tags {
            link_tags(&mut tx, pf_id, tags, data.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(pf_id)
    }
}

#[async_trait::async_trait]
impl Updatable for SqliteFeedRepository {
    type Params = IdParams;
    type Data = FeedUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.title.is_some() || data.pinned.is_some() {
            let count = {
                let (sql, values) = colette_sql::profile_feed::update(
                    params.id,
                    params.profile_id,
                    data.title,
                    data.pinned,
                )
                .build_sqlx(SqliteQueryBuilder);

                sqlx::query_with(&sql, values)
                    .execute(&mut *tx)
                    .await
                    .map(|e| e.rows_affected())
                    .map_err(|e| Error::Unknown(e.into()))?
            };
            if count == 0 {
                return Err(Error::NotFound(params.id));
            }
        }

        if let Some(tags) = data.tags {
            link_tags(&mut tx, params.id, tags, params.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for SqliteFeedRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let count = {
            let (sql, values) = colette_sql::profile_feed::delete(params.id, params.profile_id)
                .build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&self.pool)
                .await
                .map(|e| e.rows_affected())
                .map_err(|e| Error::Unknown(e.into()))?
        };
        if count == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl FeedRepository for SqliteFeedRepository {
    async fn cache(&self, data: Vec<FeedCacheData>) -> Result<(), Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        for data in data {
            create_feed_with_entries(&mut tx, data.url, data.feed)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }

    fn stream(&self) -> BoxStream<Result<String, Error>> {
        sqlx::query_scalar::<_, String>("SELECT COALESCE(url, link) FROM feeds")
            .fetch(&self.pool)
            .map_err(|e| Error::Unknown(e.into()))
            .boxed()
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct FeedSelect {
    pub id: Uuid,
    pub link: String,
    pub title: Option<String>,
    pub pinned: bool,
    pub original_title: String,
    pub url: Option<String>,
    pub tags: Option<Json<Vec<colette_core::Tag>>>,
    pub unread_count: i64,
}

impl From<FeedSelect> for colette_core::Feed {
    fn from(value: FeedSelect) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            pinned: value.pinned,
            original_title: value.original_title,
            url: value.url,
            tags: value.tags.map(|e| e.0.into_iter().collect()),
            unread_count: Some(value.unread_count),
        }
    }
}

pub(crate) async fn link_tags(
    conn: &mut SqliteConnection,
    profile_feed_id: Uuid,
    tags: TagsLinkData,
    profile_id: Uuid,
) -> sqlx::Result<()> {
    if let TagsLinkAction::Remove = tags.action {
        let (sql, values) =
            colette_sql::profile_feed_tag::delete_many_in_titles(&tags.data, profile_id)
                .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;

        return Ok(());
    }

    if let TagsLinkAction::Set = tags.action {
        let (sql, values) =
            colette_sql::profile_feed_tag::delete_many_not_in_titles(&tags.data, profile_id)
                .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;
    }

    {
        let (sql, values) = colette_sql::tag::insert_many(
            tags.data
                .iter()
                .map(|e| colette_sql::tag::InsertMany {
                    id: Some(Uuid::new_v4()),
                    title: e.to_owned(),
                })
                .collect(),
            profile_id,
        )
        .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;
    }

    let tag_ids = {
        let (sql, values) = colette_sql::tag::select_ids_by_titles(&tags.data, profile_id)
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
            .fetch_all(&mut *conn)
            .await?
    };

    {
        let insert_many = tag_ids
            .into_iter()
            .map(|e| colette_sql::profile_feed_tag::InsertMany {
                profile_feed_id,
                tag_id: e,
            })
            .collect::<Vec<_>>();

        let (sql, values) = colette_sql::profile_feed_tag::insert_many(insert_many, profile_id)
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;
    }

    Ok(())
}

pub(crate) async fn create_feed_with_entries(
    conn: &mut SqliteConnection,
    url: String,
    feed: ProcessedFeed,
) -> Result<i32, sqlx::Error> {
    let feed_id = {
        let link = feed.link.to_string();
        let url = if url == link { None } else { Some(url) };

        let (sql, values) =
            colette_sql::feed::insert(link, feed.title, url).build_sqlx(SqliteQueryBuilder);

        sqlx::query_scalar_with::<_, i32, _>(&sql, values)
            .fetch_one(&mut *conn)
            .await?
    };

    if !feed.entries.is_empty() {
        let insert_many = feed
            .entries
            .into_iter()
            .map(|e| colette_sql::feed_entry::InsertMany {
                link: e.link.to_string(),
                title: e.title,
                published_at: e.published,
                description: e.description,
                author: e.author,
                thumbnail_url: e.thumbnail.map(String::from),
            })
            .collect::<Vec<_>>();

        let (sql, values) = colette_sql::feed_entry::insert_many(insert_many, feed_id)
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;
    }

    Ok(feed_id)
}
