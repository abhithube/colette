use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, TagsLinkAction, TagsLinkData, Updatable},
    feed::{
        Cursor, Error, FeedCacheData, FeedCreateData, FeedFindManyFilters, FeedRepository,
        FeedUpdateData, ProcessedFeed,
    },
    Feed,
};
use deadpool_postgres::{GenericClient, Pool};
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use sea_query::{Expr, ExprTrait, PostgresQueryBuilder};
use sea_query_postgres::PostgresBinder;
use tokio_postgres::{types::Json, Row};
use uuid::Uuid;

pub struct PostgresFeedRepository {
    pub(crate) pool: Pool,
}

impl PostgresFeedRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresFeedRepository {
    type Params = IdParams;
    type Output = Result<Feed, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        find_by_id(&client, params).await
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresFeedRepository {
    type Data = FeedCreateData;
    type Output = Result<Feed, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let feed_id = if let Some(feed) = data.feed {
            create_feed_with_entries(&tx, data.url, feed)
                .await
                .map_err(|e| Error::Unknown(e.into()))?
        } else {
            let (sql, values) = colette_sql::feed::select_by_url(data.url.clone())
                .build_postgres(PostgresQueryBuilder);

            if let Some(row) = tx
                .query_opt(&sql, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?
            {
                row.get("id")
            } else {
                return Err(Error::Conflict(data.url));
            }
        };

        let pf_id = {
            let (mut sql, mut values) =
                colette_sql::profile_feed::select_by_unique_index(data.profile_id, feed_id)
                    .build_postgres(PostgresQueryBuilder);

            if let Some(row) = tx
                .query_opt(&sql, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?
            {
                row.get("id")
            } else {
                let id = Uuid::new_v4();

                (sql, values) =
                    colette_sql::profile_feed::insert(id, None, feed_id, data.profile_id)
                        .build_postgres(PostgresQueryBuilder);

                tx.execute(&sql, &values.as_params())
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;

                id
            }
        };

        let fe_ids = {
            let (sql, values) = colette_sql::feed_entry::select_many_by_feed_id(feed_id)
                .build_postgres(PostgresQueryBuilder);

            let mut ids: Vec<i32> = Vec::new();
            for row in tx
                .query(&sql, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?
            {
                ids.push(row.get("id"));
            }

            ids
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
                    .build_postgres(PostgresQueryBuilder);

            tx.execute(&sql, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        if let Some(tags) = data.tags {
            link_tags(&tx, pf_id, tags, data.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        let feed = find_by_id(&tx, IdParams::new(pf_id, data.profile_id)).await?;

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
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tx = client
            .transaction()
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
                .build_postgres(PostgresQueryBuilder);

                tx.execute(&sql, &values.as_params())
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
            };
            if count == 0 {
                return Err(Error::NotFound(params.id));
            }
        }

        if let Some(tags) = data.tags {
            link_tags(&tx, params.id, tags, params.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        let feed = find_by_id(&tx, params).await?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(feed)
    }
}

#[async_trait::async_trait]
impl Deletable for PostgresFeedRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let count = {
            let (sql, values) = colette_sql::profile_feed::delete(params.id, params.profile_id)
                .build_postgres(PostgresQueryBuilder);

            client
                .execute(&sql, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?
        };
        if count == 0 {
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
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        find(&client, None, profile_id, limit, cursor, filters).await
    }

    async fn cache(&self, data: FeedCacheData) -> Result<(), Error> {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        create_feed_with_entries(&tx, data.url, data.feed)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }

    async fn stream(&self) -> Result<BoxStream<Result<String, Error>>, Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        client
            .query_raw::<_, _, &[&str; 0]>("SELECT COALESCE(url, link) AS url FROM feed", &[])
            .await
            .map(|e| {
                e.map(|e| e.map(|e| e.get("url")))
                    .map_err(|e| Error::Unknown(e.into()))
                    .boxed()
            })
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[derive(Debug, Clone)]
struct FeedSelect(Feed);

impl From<&Row> for FeedSelect {
    fn from(value: &Row) -> Self {
        Self(Feed {
            id: value.get("id"),
            link: value.get("link"),
            title: value.get("title"),
            pinned: value.get("pinned"),
            original_title: value.get("original_title"),
            url: value.get("url"),
            tags: value
                .get::<_, Option<Json<Vec<colette_core::Tag>>>>("tags")
                .map(|e| e.0),
            unread_count: Some(value.get("unread_count")),
        })
    }
}

pub(crate) async fn find<C: GenericClient>(
    client: &C,
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
        Expr::cust_with_expr(
            r#"EXISTS (SELECT 1 FROM JSONB_ARRAY_ELEMENTS("json_tags"."tags") AS "t" WHERE ?)"#,
            Expr::cust(r#""t" ->> 'title'"#).is_in(e),
        )
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
    .build_postgres(PostgresQueryBuilder);

    client
        .query(&sql, &values.as_params())
        .await
        .map(|e| {
            e.into_iter()
                .map(|e| FeedSelect::from(&e).0)
                .collect::<Vec<_>>()
        })
        .map_err(|e| Error::Unknown(e.into()))
}

async fn find_by_id<C: GenericClient>(client: &C, params: IdParams) -> Result<Feed, Error> {
    let mut feeds = find(client, Some(params.id), params.profile_id, None, None, None).await?;
    if feeds.is_empty() {
        return Err(Error::NotFound(params.id));
    }

    Ok(feeds.swap_remove(0))
}

pub(crate) async fn create_feed_with_entries<C: GenericClient>(
    client: &C,
    url: String,
    feed: ProcessedFeed,
) -> Result<i32, tokio_postgres::Error> {
    let link = feed.link.to_string();
    let url = if url == link { None } else { Some(url) };

    let feed_id = {
        let (sql, values) =
            colette_sql::feed::insert(link, feed.title, url).build_postgres(PostgresQueryBuilder);

        let row = client.query_one(&sql, &values.as_params()).await?;

        row.get::<_, i32>("id")
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
            .build_postgres(PostgresQueryBuilder);

        client.execute(&sql, &values.as_params()).await?;
    }

    Ok(feed_id)
}

pub(crate) async fn link_tags<C: GenericClient>(
    client: &C,
    profile_feed_id: Uuid,
    tags: TagsLinkData,
    profile_id: Uuid,
) -> Result<(), tokio_postgres::Error> {
    if let TagsLinkAction::Remove = tags.action {
        let (sql, values) =
            colette_sql::profile_feed_tag::delete_many_in_titles(&tags.data, profile_id)
                .build_postgres(PostgresQueryBuilder);

        client.execute(&sql, &values.as_params()).await?;

        return Ok(());
    }

    if let TagsLinkAction::Set = tags.action {
        let (sql, values) =
            colette_sql::profile_feed_tag::delete_many_not_in_titles(&tags.data, profile_id)
                .build_postgres(PostgresQueryBuilder);

        client.execute(&sql, &values.as_params()).await?;
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
        .build_postgres(PostgresQueryBuilder);

        client.execute(&sql, &values.as_params()).await?;
    }

    let tag_ids = {
        let (sql, values) = colette_sql::tag::select_ids_by_titles(&tags.data, profile_id)
            .build_postgres(PostgresQueryBuilder);

        let mut ids: Vec<Uuid> = Vec::new();
        for row in client.query(&sql, &values.as_params()).await? {
            ids.push(row.get("id"));
        }

        ids
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
            .build_postgres(PostgresQueryBuilder);

        client.execute(&sql, &values.as_params()).await?;
    }

    Ok(())
}
