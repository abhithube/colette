use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, TagsLinkAction, TagsLinkData, Updatable},
    feed::{
        Cursor, Error, FeedCacheData, FeedCreateData, FeedFindManyFilters, FeedRepository,
        FeedUpdateData, ProcessedFeed,
    },
    Feed,
};
use deadpool_sqlite::Pool;
use futures::{
    stream::{self, BoxStream},
    StreamExt,
};
use rusqlite::{types::Value, Connection, OptionalExtension, Row};
use sea_query::{Expr, ExprTrait, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use uuid::Uuid;

pub struct SqliteFeedRepository {
    pool: Pool,
}

impl SqliteFeedRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteFeedRepository {
    type Params = IdParams;
    type Output = Result<Feed, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| find_by_id(conn, params.id, params.profile_id))
            .await
            .unwrap()
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })
    }
}

#[async_trait::async_trait]
impl Creatable for SqliteFeedRepository {
    type Data = FeedCreateData;
    type Output = Result<Feed, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let url = data.url.clone();

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            let feed_id = if let Some(feed) = data.feed {
                create_feed_with_entries(&tx, data.url, feed)
            } else {
                let (sql, values) = colette_sql::feed::select_by_url(data.url.clone())
                    .build_rusqlite(SqliteQueryBuilder);

                if let Some(id) = tx
                    .prepare_cached(&sql)?
                    .query_row(&*values.as_params(), |row| row.get::<_, i32>("id"))
                    .optional()?
                {
                    Ok(id)
                } else {
                    Err(rusqlite::Error::SqliteFailure(
                        rusqlite::ffi::Error {
                            code: rusqlite::ErrorCode::ConstraintViolation,
                            extended_code: rusqlite::ffi::SQLITE_CONSTRAINT,
                        },
                        None,
                    ))
                }
            }?;

            let pf_id = {
                let (mut sql, mut values) =
                    colette_sql::profile_feed::select_by_unique_index(data.profile_id, feed_id)
                        .build_rusqlite(SqliteQueryBuilder);

                if let Some(id) = tx
                    .prepare_cached(&sql)?
                    .query_row(&*values.as_params(), |row| row.get::<_, Uuid>("id"))
                    .optional()?
                {
                    id
                } else {
                    let id = Uuid::new_v4();

                    (sql, values) =
                        colette_sql::profile_feed::insert(id, None, feed_id, data.profile_id)
                            .build_rusqlite(SqliteQueryBuilder);

                    tx.prepare_cached(&sql)?.execute(&*values.as_params())?;

                    id
                }
            };

            let fe_ids = {
                let (sql, values) = colette_sql::feed_entry::select_many_by_feed_id(feed_id)
                    .build_rusqlite(SqliteQueryBuilder);

                let mut ids: Vec<i32> = Vec::new();

                let mut stmt = tx.prepare_cached(&sql)?;
                let mut rows = stmt.query(&*values.as_params())?;

                while let Some(row) = rows.next()? {
                    ids.push(row.get("id")?);
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
                let (sql, values) = colette_sql::profile_feed_entry::insert_many(
                    insert_many,
                    pf_id,
                    data.profile_id,
                )
                .build_rusqlite(SqliteQueryBuilder);

                tx.prepare_cached(&sql)?.execute(&*values.as_params())?;
            }

            if let Some(tags) = data.tags {
                link_tags(&tx, pf_id, tags, data.profile_id)?;
            }

            let feed = find_by_id(&tx, pf_id, data.profile_id)?;

            tx.commit()?;

            Ok(feed)
        })
        .await
        .unwrap()
        .map_err(|e| match e {
            rusqlite::Error::SqliteFailure(e, _)
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Error::Conflict(url)
            }
            _ => Error::Unknown(e.into()),
        })
    }
}

#[async_trait::async_trait]
impl Updatable for SqliteFeedRepository {
    type Params = IdParams;
    type Data = FeedUpdateData;
    type Output = Result<Feed, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            if data.title.is_some() || data.pinned.is_some() {
                let count = {
                    let (sql, values) = colette_sql::profile_feed::update(
                        params.id,
                        params.profile_id,
                        data.title,
                        data.pinned,
                    )
                    .build_rusqlite(SqliteQueryBuilder);

                    tx.prepare_cached(&sql)?.execute(&*values.as_params())?
                };
                if count == 0 {
                    return Err(rusqlite::Error::QueryReturnedNoRows);
                }
            }

            if let Some(tags) = data.tags {
                link_tags(&tx, params.id, tags, params.profile_id)?;
            }

            let feed = find_by_id(&tx, params.id, params.profile_id)?;

            tx.commit()?;

            Ok(feed)
        })
        .await
        .unwrap()
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }
}

#[async_trait::async_trait]
impl Deletable for SqliteFeedRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let (sql, values) = colette_sql::profile_feed::delete(params.id, params.profile_id)
                .build_rusqlite(SqliteQueryBuilder);

            let count = conn.execute(&sql, &*values.as_params())?;
            if count == 0 {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }

            Ok(())
        })
        .await
        .unwrap()
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }
}

#[async_trait::async_trait]
impl FeedRepository for SqliteFeedRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
        filters: Option<FeedFindManyFilters>,
    ) -> Result<Vec<Feed>, Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| find(conn, None, profile_id, limit, cursor, filters))
            .await
            .unwrap()
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn cache(&self, data: FeedCacheData) -> Result<(), Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            create_feed_with_entries(&tx, data.url, data.feed)?;

            tx.commit()
        })
        .await
        .unwrap()
        .map_err(|e| Error::Unknown(e.into()))
    }

    async fn stream(&self) -> Result<BoxStream<Result<String, Error>>, Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let mut stmt = conn.prepare_cached("SELECT COALESCE(url, link) AS url FROM feed")?;
            let rows = stmt.query_map([], |row| row.get::<_, String>("url"))?;

            Ok::<_, rusqlite::Error>(rows.into_iter().collect::<Vec<_>>())
        })
        .await
        .unwrap()
        .map(|e| {
            stream::iter(
                e.into_iter()
                    .map(|e| e.map_err(|e| Error::Unknown(e.into())))
                    .collect::<Vec<_>>(),
            )
            .boxed()
        })
        .map_err(|e| Error::Unknown(e.into()))
    }
}

#[derive(Debug, Clone)]
struct FeedSelect(Feed);

impl TryFrom<&Row<'_>> for FeedSelect {
    type Error = rusqlite::Error;

    fn try_from(value: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self(Feed {
            id: value.get("id")?,
            link: value.get("link")?,
            title: value.get("title")?,
            pinned: value.get("pinned")?,
            original_title: value.get("original_title")?,
            url: value.get("url")?,
            tags: value.get::<_, Value>("tags").map(|e| match e {
                Value::Text(text) => Some(serde_json::from_str(&text).unwrap()),
                _ => Some(Vec::new()),
            })?,
            unread_count: Some(value.get("unread_count")?),
        }))
    }
}

pub(crate) fn find(
    conn: &Connection,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<FeedFindManyFilters>,
) -> rusqlite::Result<Vec<Feed>> {
    let mut pinned: Option<bool> = None;
    let mut tags: Option<Vec<String>> = None;

    if let Some(filters) = filters {
        pinned = filters.pinned;
        tags = filters.tags;
    }

    // let jsonb_agg = Expr::cust(
    //     r#"JSON_GROUP_ARRAY(JSON_OBJECT('id', HEX("tag"."id"), 'title', "tag"."title") ORDER BY "tag"."title") FILTER (WHERE "tag"."id" IS NOT NULL)"#,
    // );
    let jsonb_agg = Expr::cust(
        r#"JSON_GROUP_ARRAY(JSON_OBJECT('id', HEX("tag"."id"), 'title', "tag"."title")) FILTER (WHERE "tag"."id" IS NOT NULL)"#,
    );

    let tags_subquery = tags.map(|e| {
        Expr::cust_with_expr(
            r#"EXISTS (SELECT 1 FROM JSON_EACH("json_tags"."tags") AS "t" WHERE ?)"#,
            Expr::cust(r#""t"."value" ->> 'title'"#).is_in(e),
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
    .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = conn.prepare_cached(&sql)?;
    let mut rows = stmt.query(&*values.as_params())?;

    let mut feeds: Vec<Feed> = Vec::new();
    while let Some(row) = rows.next()? {
        feeds.push(FeedSelect::try_from(row).map(|e| e.0)?);
    }

    Ok(feeds)
}

fn find_by_id(conn: &Connection, id: Uuid, profile_id: Uuid) -> rusqlite::Result<Feed> {
    let mut feeds = find(conn, Some(id), profile_id, None, None, None)?;
    if feeds.is_empty() {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    Ok(feeds.swap_remove(0))
}

pub(crate) fn create_feed_with_entries(
    conn: &Connection,
    url: String,
    feed: ProcessedFeed,
) -> rusqlite::Result<i32> {
    let link = feed.link.to_string();
    let url = if url == link { None } else { Some(url) };

    let feed_id = {
        let (sql, values) =
            colette_sql::feed::insert(link, feed.title, url).build_rusqlite(SqliteQueryBuilder);

        conn.prepare_cached(&sql)?
            .query_row(&*values.as_params(), |row| row.get::<_, i32>("id"))?
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
            .build_rusqlite(SqliteQueryBuilder);

        conn.execute(&sql, &*values.as_params())?;
    }

    Ok(feed_id)
}

pub(crate) fn link_tags(
    conn: &Connection,
    profile_feed_id: Uuid,
    tags: TagsLinkData,
    profile_id: Uuid,
) -> rusqlite::Result<()> {
    if let TagsLinkAction::Remove = tags.action {
        let (sql, values) =
            colette_sql::profile_feed_tag::delete_many_in_titles(&tags.data, profile_id)
                .build_rusqlite(SqliteQueryBuilder);

        conn.prepare_cached(&sql)?.execute(&*values.as_params())?;

        return Ok(());
    }

    if let TagsLinkAction::Set = tags.action {
        let (sql, values) =
            colette_sql::profile_feed_tag::delete_many_not_in_titles(&tags.data, profile_id)
                .build_rusqlite(SqliteQueryBuilder);

        conn.prepare_cached(&sql)?.execute(&*values.as_params())?;
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
        .build_rusqlite(SqliteQueryBuilder);

        conn.prepare_cached(&sql)?.execute(&*values.as_params())?;
    }

    let tag_ids = {
        let (sql, values) = colette_sql::tag::select_ids_by_titles(&tags.data, profile_id)
            .build_rusqlite(SqliteQueryBuilder);

        let mut ids: Vec<Uuid> = Vec::new();

        let mut stmt = conn.prepare_cached(&sql)?;
        let mut rows = stmt.query(&*values.as_params())?;

        while let Some(row) = rows.next()? {
            ids.push(row.get("id")?);
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
            .build_rusqlite(SqliteQueryBuilder);

        conn.prepare_cached(&sql)?.execute(&*values.as_params())?;
    }

    Ok(())
}
