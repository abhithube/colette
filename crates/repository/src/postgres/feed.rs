use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    feed::{
        Error, FeedCacheData, FeedCreateData, FeedFindParams, FeedRepository, FeedUpdateData,
        ProcessedFeed,
    },
    Feed,
};
use deadpool_postgres::{
    tokio_postgres::{
        self,
        types::{Json, Type},
        Row,
    },
    GenericClient, Pool,
};
use futures::{
    stream::{self, BoxStream},
    StreamExt,
};
use sea_query::{Expr, ExprTrait, PostgresQueryBuilder, WithQuery};
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresFeedRepository {
    pool: Pool,
}

impl PostgresFeedRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresFeedRepository {
    type Params = FeedFindParams;
    type Output = Result<Vec<Feed>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) = build_select(params).build_postgres(PostgresQueryBuilder);

        let stmt = client
            .prepare_cached(&sql)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        client
            .query(&stmt, &values.as_params())
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
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let feed_id = {
            let (sql, values) =
                crate::feed::select_by_url(data.url.clone()).build_postgres(PostgresQueryBuilder);

            let stmt = tx
                .prepare_cached(&sql)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            if let Some(row) = tx
                .query_opt(&stmt, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?
            {
                Ok(row.get::<_, Uuid>("id"))
            } else {
                Err(Error::Conflict(data.url))
            }
        }?;

        let pf_id = {
            let (mut sql, mut values) =
                crate::user_feed::select_by_unique_index(data.user_id, feed_id)
                    .build_postgres(PostgresQueryBuilder);

            let stmt = tx
                .prepare_cached(&sql)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            if let Some(row) = tx
                .query_opt(&stmt, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?
            {
                row.get::<_, Uuid>("id")
            } else {
                (sql, values) = crate::user_feed::insert(
                    None,
                    data.title,
                    data.folder_id,
                    feed_id,
                    data.user_id,
                )
                .build_postgres(PostgresQueryBuilder);

                let stmt = tx
                    .prepare_cached(&sql)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;

                tx.query_one(&stmt, &values.as_params())
                    .await
                    .map(|e| e.get::<_, Uuid>("id"))
                    .map_err(|e| Error::Unknown(e.into()))?
            }
        };

        link_entries_to_users(&tx, feed_id)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if let Some(tags) = data.tags {
            link_tags(&tx, pf_id, tags, data.user_id)
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
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.title.is_some() {
            let (sql, values) =
                crate::user_feed::update(params.id, params.user_id, data.title, data.folder_id)
                    .build_postgres(PostgresQueryBuilder);

            let stmt = tx
                .prepare_cached(&sql)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            let count = tx
                .execute(&stmt, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
            if count == 0 {
                return Err(Error::NotFound(params.id));
            }
        }

        if let Some(tags) = data.tags {
            link_tags(&tx, params.id, tags, params.user_id)
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
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) = crate::user_feed::delete(params.id, params.user_id)
            .build_postgres(PostgresQueryBuilder);

        let stmt = client
            .prepare_cached(&sql)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let count = client
            .execute(&stmt, &values.as_params())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        if count == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl FeedRepository for PostgresFeedRepository {
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

    async fn stream(&self) -> Result<BoxStream<String>, Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) = crate::feed::iterate().build_postgres(PostgresQueryBuilder);

        let stmt = client
            .prepare_cached(&sql)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let rows = client
            .query(&stmt, &values.as_params())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        let urls = stream::iter(rows)
            .map(|row| row.get::<_, String>("url"))
            .boxed();

        Ok(urls)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FeedSelect(pub(crate) Feed);

impl From<Row> for FeedSelect {
    fn from(value: Row) -> Self {
        Self(Feed {
            id: value.get("id"),
            link: value.get("link"),
            title: value.get("title"),
            xml_url: value.get("xml_url"),
            original_title: value.get("original_title"),
            folder_id: value.get("folder_id"),
            tags: value
                .get::<_, Option<Json<Vec<colette_core::Tag>>>>("tags")
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

pub(crate) async fn create_feed_with_entries<C: GenericClient>(
    client: &C,
    url: String,
    feed: ProcessedFeed,
) -> Result<Uuid, tokio_postgres::Error> {
    let feed_id = {
        let link = feed.link.to_string();
        let xml_url = if url == link { None } else { Some(url) };

        let (sql, values) =
            crate::feed::insert(link, feed.title, xml_url).build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;

        client
            .query_one(&stmt, &values.as_params())
            .await
            .map(|e| e.get::<_, Uuid>("id"))?
    };

    if !feed.entries.is_empty() {
        let insert_many = feed
            .entries
            .into_iter()
            .map(|e| crate::feed_entry::InsertMany {
                link: e.link.to_string(),
                title: e.title,
                published_at: e.published,
                description: e.description,
                author: e.author,
                thumbnail_url: e.thumbnail.map(String::from),
            })
            .collect::<Vec<_>>();

        let (sql, values) = crate::feed_entry::insert_many(&insert_many, feed_id)
            .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;

        client.execute(&stmt, &values.as_params()).await?;
    }

    Ok(feed_id)
}

pub(crate) async fn link_entries_to_users<C: GenericClient>(
    client: &C,
    feed_id: Uuid,
) -> Result<(), tokio_postgres::Error> {
    let fe_ids = {
        let (sql, values) =
            crate::feed_entry::select_many_by_feed_id(feed_id).build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;

        client.query(&stmt, &values.as_params()).await.map(|e| {
            e.into_iter()
                .map(|e| e.get::<_, Uuid>("id"))
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
                .build_postgres(PostgresQueryBuilder);

        let mut types: Vec<Type> = Vec::new();
        for _ in insert_many.iter() {
            types.push(Type::UUID);
        }

        let stmt = client.prepare_typed_cached(&sql, &types).await?;

        client.execute(&stmt, &values.as_params()).await?;
    }

    Ok(())
}

pub(crate) async fn link_tags<C: GenericClient>(
    client: &C,
    user_feed_id: Uuid,
    tags: Vec<String>,
    user_id: Uuid,
) -> Result<(), tokio_postgres::Error> {
    {
        let (sql, values) = crate::user_feed_tag::delete_many_not_in_titles(&tags, user_id)
            .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;

        client.execute(&stmt, &values.as_params()).await?;
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
            crate::tag::insert_many(&insert_many, user_id).build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;

        client.execute(&stmt, &values.as_params()).await?;
    }

    {
        let (sql, values) =
            crate::tag::select_ids_by_titles(&tags, user_id).build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;

        let tag_ids = client.query(&stmt, &values.as_params()).await.map(|e| {
            e.into_iter()
                .map(|e| e.get::<_, Uuid>("id"))
                .collect::<Vec<_>>()
        })?;

        let insert_many = tag_ids
            .into_iter()
            .map(|e| crate::user_feed_tag::InsertMany {
                user_feed_id,
                tag_id: e,
            })
            .collect::<Vec<_>>();

        let (sql, values) = crate::user_feed_tag::insert_many(&insert_many, user_id)
            .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;

        client.execute(&stmt, &values.as_params()).await?;
    }

    Ok(())
}
