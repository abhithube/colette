use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    feed::{
        ConflictError, Error, FeedCacheData, FeedCreateData, FeedFindParams, FeedRepository,
        FeedUpdateData, ProcessedFeed,
    },
    Feed,
};
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use sea_query::{Expr, ExprTrait, PostgresQueryBuilder, WithQuery};
use sea_query_binder::SqlxBinder;
use sqlx::{postgres::PgRow, types::Json, PgConnection, Pool, Postgres, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresFeedRepository {
    pool: Pool<Postgres>,
}

impl PostgresFeedRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresFeedRepository {
    type Params = FeedFindParams;
    type Output = Result<Vec<Feed>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let (sql, values) = build_select(params).build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .fetch_all(&self.pool)
            .await
            .map(|e| {
                e.into_iter()
                    .map(|e| FeedSelect::from(e).0)
                    .collect::<Vec<_>>()
            })
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresFeedRepository {
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
                crate::feed::select_by_url(data.url.clone()).build_sqlx(PostgresQueryBuilder);

            if let Some(id) = sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?
            {
                Ok(id)
            } else {
                Err(Error::Conflict(ConflictError::NotCached(data.url.clone())))
            }
        }?;

        let pf_id = {
            let (mut sql, mut values) =
                crate::user_feed::select_by_unique_index(data.user_id, feed_id)
                    .build_sqlx(PostgresQueryBuilder);

            if let Some(id) = sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?
            {
                id
            } else {
                (sql, values) = crate::user_feed::insert(
                    None,
                    data.title,
                    data.folder_id,
                    feed_id,
                    data.user_id,
                )
                .build_sqlx(PostgresQueryBuilder);

                sqlx::query_with(&sql, values)
                    .fetch_one(&mut *tx)
                    .await
                    .map(|e| e.get::<Uuid, _>("id"))
                    .map_err(|e| match e {
                        sqlx::Error::Database(e) if e.is_unique_violation() => {
                            Error::Conflict(ConflictError::AlreadyExists(data.url))
                        }
                        _ => Error::Unknown(e.into()),
                    })?
            }
        };

        link_entries_to_users(&mut tx, feed_id)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if let Some(tags) = data.tags {
            link_tags(&mut tx, pf_id, &tags, data.user_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(pf_id)
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresFeedRepository {
    type Params = IdParams;
    type Data = FeedUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.title.is_some() {
            let (sql, values) =
                crate::user_feed::update(params.id, params.user_id, data.title, data.folder_id)
                    .build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&mut *tx)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound(params.id),
                    _ => Error::Unknown(e.into()),
                })?;
        }

        if let Some(tags) = data.tags {
            link_tags(&mut tx, params.id, &tags, params.user_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for PostgresFeedRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let (sql, values) =
            crate::user_feed::delete(params.id, params.user_id).build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl FeedRepository for PostgresFeedRepository {
    async fn cache(&self, data: FeedCacheData) -> Result<(), Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        create_feed_with_entries(&mut tx, data.url, data.feed)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }

    fn stream(&self) -> BoxStream<Result<String, Error>> {
        sqlx::query_scalar::<_, String>("SELECT coalesce(xml_url, link) AS url FROM feeds JOIN user_feeds ON user_feeds.feed_id = feeds.id")
            .fetch(&self.pool)
            .map_err(|e| Error::Unknown(e.into()))
            .boxed()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FeedSelect(pub(crate) Feed);

impl From<PgRow> for FeedSelect {
    fn from(value: PgRow) -> Self {
        Self(Feed {
            id: value.get("id"),
            link: value.get("link"),
            title: value.get("title"),
            xml_url: value.get("xml_url"),
            original_title: value.get("original_title"),
            folder_id: value.get("folder_id"),
            tags: value
                .get::<Option<Json<Vec<colette_core::Tag>>>, _>("tags")
                .map(|e| e.0),
            unread_count: Some(value.get("unread_count")),
        })
    }
}

pub(crate) fn build_select(params: FeedFindParams) -> WithQuery {
    let jsonb_agg = Expr::cust(
        r#"JSONB_AGG(JSONB_BUILD_OBJECT('id', "tags"."id", 'title', "tags"."title") ORDER BY "tags"."title") FILTER (WHERE "tags"."id" IS NOT NULL)"#,
    );

    let tags_subquery = params.tags.map(|e| {
        Expr::cust_with_expr(
            r#"EXISTS (SELECT 1 FROM JSONB_ARRAY_ELEMENTS("json_tags"."tags") AS "t" WHERE ?)"#,
            Expr::cust(r#""t" ->> 'title'"#).is_in(e),
        )
    });

    crate::user_feed::select(
        params.id,
        params.folder_id,
        params.user_id,
        params.cursor,
        params.limit,
        jsonb_agg,
        tags_subquery,
    )
}

pub(crate) async fn create_feed_with_entries(
    conn: &mut PgConnection,
    url: String,
    feed: ProcessedFeed,
) -> Result<Uuid, sqlx::Error> {
    let feed_id = {
        let link = feed.link.to_string();
        let xml_url = if url == link { None } else { Some(url) };

        let (sql, values) =
            crate::feed::insert(None, link, feed.title, xml_url).build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .fetch_one(&mut *conn)
            .await
            .map(|e| e.get::<Uuid, _>("id"))?
    };

    if !feed.entries.is_empty() {
        let insert_many = feed
            .entries
            .into_iter()
            .map(|e| crate::feed_entry::InsertMany {
                id: None,
                link: e.link.to_string(),
                title: e.title,
                published_at: e.published,
                description: e.description,
                author: e.author,
                thumbnail_url: e.thumbnail.map(String::from),
            })
            .collect::<Vec<_>>();

        let (sql, values) =
            crate::feed_entry::insert_many(&insert_many, feed_id).build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;
    }

    Ok(feed_id)
}

pub(crate) async fn link_entries_to_users(
    conn: &mut PgConnection,
    feed_id: Uuid,
) -> Result<(), sqlx::Error> {
    let fe_ids = {
        let (sql, values) =
            crate::feed_entry::select_many_by_feed_id(feed_id).build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .fetch_all(&mut *conn)
            .await
            .map(|e| {
                e.into_iter()
                    .map(|e| e.get::<Uuid, _>("id"))
                    .collect::<Vec<_>>()
            })?
    };

    if !fe_ids.is_empty() {
        let insert_many = fe_ids
            .into_iter()
            .map(|feed_entry_id| crate::user_feed_entry::InsertMany {
                id: None,
                feed_entry_id,
            })
            .collect::<Vec<_>>();

        let (sql, values) =
            crate::user_feed_entry::insert_many_for_all_users(&insert_many, feed_id)
                .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;
    }

    Ok(())
}

pub(crate) async fn link_tags(
    conn: &mut PgConnection,
    user_feed_id: Uuid,
    tags: &[String],
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    {
        let (sql, values) = crate::user_feed_tag::delete_many_not_in_titles(tags, user_id)
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;
    }

    {
        let insert_many = tags
            .iter()
            .map(|e| crate::tag::InsertMany {
                id: Some(Uuid::new_v4()),
                title: e.to_owned(),
            })
            .collect::<Vec<_>>();

        let (sql, values) =
            crate::tag::insert_many(&insert_many, user_id).build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;
    }

    {
        let (sql, values) = crate::user_feed_tag::insert_many(user_feed_id, tags, user_id)
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;
    }

    Ok(())
}
