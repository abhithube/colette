use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, TagsLinkAction, TagsLinkData, Updatable},
    feed::{
        Cursor, Error, FeedCacheData, FeedCreateData, FeedFindManyFilters, FeedRepository,
        FeedUpdateData, ProcessedFeed,
    },
    Feed,
};
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use sea_query::{Expr, PostgresQueryBuilder};
use sea_query_binder::SqlxBinder;
use sqlx::{
    types::{Json, Uuid},
    PgExecutor, PgPool,
};

pub struct PostgresFeedRepository {
    pub(crate) pool: PgPool,
}

impl PostgresFeedRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresFeedRepository {
    type Params = IdParams;
    type Output = Result<Feed, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        find_by_id(&self.pool, params).await
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresFeedRepository {
    type Data = FeedCreateData;
    type Output = Result<Feed, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let feed_id = if let Some(feed) = data.feed {
            create_feed_with_entries(&mut tx, data.url, feed).await
        } else {
            let (sql, values) =
                colette_sql::feed::select_by_url(data.url.clone()).build_sqlx(PostgresQueryBuilder);

            sqlx::query_scalar_with::<_, i32, _>(&sql, values)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::Conflict(data.url),
                    _ => Error::Unknown(e.into()),
                })
        }?;

        let pf_id = {
            let (mut sql, mut values) =
                colette_sql::profile_feed::select_by_unique_index(data.profile_id, feed_id)
                    .build_sqlx(PostgresQueryBuilder);

            if let Some(id) = sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?
            {
                Ok(id)
            } else {
                (sql, values) = colette_sql::profile_feed::insert(
                    Uuid::new_v4(),
                    None,
                    feed_id,
                    data.profile_id,
                )
                .build_sqlx(PostgresQueryBuilder);

                sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))
            }?
        };

        let fe_ids = {
            let (sql, values) = colette_sql::feed_entry::select_many_by_feed_id(feed_id)
                .build_sqlx(PostgresQueryBuilder);

            sqlx::query_scalar_with::<_, i32, _>(&sql, values)
                .fetch_all(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?
        };

        let insert_many = fe_ids
            .into_iter()
            .map(
                |feed_entry_id| colette_sql::profile_feed_entry::InsertMany {
                    id: Uuid::new_v4(),
                    feed_entry_id,
                },
            )
            .collect::<Vec<_>>();

        {
            let (sql, values) =
                colette_sql::profile_feed_entry::insert_many(insert_many, pf_id, data.profile_id)
                    .build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        if let Some(tags) = data.tags {
            link_tags(&mut tx, pf_id, tags, data.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        let feed = find_by_id(&mut *tx, IdParams::new(pf_id, data.profile_id)).await?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(feed)
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresFeedRepository {
    type Params = IdParams;
    type Data = FeedUpdateData;
    type Output = Result<Feed, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.title.is_some() || data.pinned.is_some() {
            let result = {
                let (sql, values) = colette_sql::profile_feed::update(
                    params.id,
                    params.profile_id,
                    data.title,
                    data.pinned,
                )
                .build_sqlx(PostgresQueryBuilder);

                sqlx::query_with(&sql, values)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
            };
            if result.rows_affected() == 0 {
                return Err(Error::NotFound(params.id));
            }
        }

        if let Some(tags) = data.tags {
            link_tags(&mut tx, params.id, tags, params.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        let feed = find_by_id(&mut *tx, params).await?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(feed)
    }
}

#[async_trait::async_trait]
impl Deletable for PostgresFeedRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let result = {
            let (sql, values) = colette_sql::profile_feed::delete(params.id, params.profile_id)
                .build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::Unknown(e.into()))?
        };
        if result.rows_affected() == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl FeedRepository for PostgresFeedRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
        filters: Option<FeedFindManyFilters>,
    ) -> Result<Vec<Feed>, Error> {
        find(&self.pool, None, profile_id, limit, cursor, filters).await
    }

    async fn cache(&self, data: FeedCacheData) -> Result<(), Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        create_feed_with_entries(&mut tx, data.url, data.feed).await?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }

    fn stream(&self) -> BoxStream<Result<(i32, String), Error>> {
        sqlx::query_as::<_, (i32, String)>("SELECT id, COALESCE(url, link) FROM feed")
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
    pub tags: Option<Json<Vec<TagSelect>>>,
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
            tags: value
                .tags
                .map(|e| e.0.into_iter().map(|e| e.into()).collect()),
            unread_count: Some(value.unread_count),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TagSelect {
    id: Uuid,
    title: String,
}

impl From<TagSelect> for colette_core::Tag {
    fn from(value: TagSelect) -> Self {
        Self {
            id: value.id,
            title: value.title,
            bookmark_count: None,
            feed_count: None,
        }
    }
}

pub(crate) async fn find(
    executor: impl PgExecutor<'_>,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<FeedFindManyFilters>,
) -> Result<Vec<Feed>, Error> {
    let mut pinned: Option<bool> = None;
    let mut tags: Option<Vec<String>> = None;

    if let Some(filters) = filters {
        pinned = filters.pinned;
        tags = filters.tags;
    }

    let jsonb_agg = Expr::cust(
        r#"JSONB_AGG(JSONB_BUILD_OBJECT('id', "tag"."id", 'title', "tag"."title") ORDER BY "tag"."title") FILTER (WHERE "tag"."id" IS NOT NULL)"#,
    );

    let tags_subquery = tags.map(|e| {
        Expr::cust_with_expr(r#"EXISTS (SELECT 1 FROM JSONB_ARRAY_ELEMENTS("json_tags"."tags") AS "t" WHERE "t" ->> 'title' = ANY($1))"#, e)
    });

    let (sql, values) = colette_sql::profile_feed::select(
        id,
        profile_id,
        pinned,
        cursor,
        limit,
        jsonb_agg,
        tags_subquery,
    )
    .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with::<_, FeedSelect, _>(&sql, values)
        .fetch_all(executor)
        .await
        .map(|e| e.into_iter().map(Feed::from).collect())
        .map_err(|e| Error::Unknown(e.into()))
}

async fn find_by_id(executor: impl PgExecutor<'_>, params: IdParams) -> Result<Feed, Error> {
    let mut feeds = find(
        executor,
        Some(params.id),
        params.profile_id,
        None,
        None,
        None,
    )
    .await?;
    if feeds.is_empty() {
        return Err(Error::NotFound(params.id));
    }

    Ok(feeds.swap_remove(0))
}

async fn create_feed_with_entries(
    conn: &mut sqlx::PgConnection,
    url: String,
    feed: ProcessedFeed,
) -> Result<i32, Error> {
    let link = feed.link.to_string();
    let url = if url == link { None } else { Some(url) };

    let feed_id = {
        let (sql, values) =
            colette_sql::feed::insert(link, feed.title, url).build_sqlx(PostgresQueryBuilder);

        sqlx::query_scalar_with::<_, i32, _>(&sql, values)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| Error::Unknown(e.into()))?
    };

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

    {
        let (sql, values) = colette_sql::feed_entry::insert_many(insert_many, feed_id)
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&mut *conn)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
    }

    Ok(feed_id)
}

pub(crate) async fn link_tags(
    conn: &mut sqlx::PgConnection,
    profile_feed_id: Uuid,
    tags: TagsLinkData,
    profile_id: Uuid,
) -> sqlx::Result<()> {
    if let TagsLinkAction::Remove = tags.action {
        let (sql, values) =
            colette_sql::profile_feed_tag::delete_many_in_titles(&tags.data, profile_id)
                .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;

        return Ok(());
    }

    if let TagsLinkAction::Set = tags.action {
        let (sql, values) =
            colette_sql::profile_feed_tag::delete_many_not_in_titles(&tags.data, profile_id)
                .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;
    }

    {
        let (sql, values) = colette_sql::tag::insert_many(
            tags.data
                .iter()
                .map(|e| colette_sql::tag::InsertMany {
                    id: Uuid::new_v4(),
                    title: e.to_owned(),
                })
                .collect(),
            profile_id,
        )
        .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;
    }

    let tag_ids = {
        let (sql, values) = colette_sql::tag::select_ids_by_titles(&tags.data, profile_id)
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
            .fetch_all(&mut *conn)
            .await?
    };

    let insert_many = tag_ids
        .into_iter()
        .map(|e| colette_sql::profile_feed_tag::InsertMany {
            profile_feed_id,
            tag_id: e,
        })
        .collect::<Vec<_>>();

    {
        let (sql, values) = colette_sql::profile_feed_tag::insert_many(insert_many, profile_id)
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;
    }

    Ok(())
}
