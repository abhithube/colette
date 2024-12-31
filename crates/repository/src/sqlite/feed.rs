use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    feed::{
        Error, FeedCacheData, FeedCreateData, FeedFindParams, FeedRepository, FeedUpdateData,
        ProcessedFeed,
    },
    Feed,
};
use deadpool_sqlite::{
    rusqlite::{self, types::Value, Connection, OptionalExtension, Row},
    Pool,
};
use futures::{stream::{self, BoxStream}, StreamExt};
use sea_query::{Expr, ExprTrait, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use uuid::Uuid;

#[derive(Debug, Clone)]
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
    type Params = FeedFindParams;
    type Output = Result<Vec<Feed>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        
        let conn = self
        .pool
        .get()
        .await
        .map_err(|e| Error::Unknown(e.into()))?;
    
    conn.interact(move |conn| {
            let jsonb_agg = Expr::cust(
                r#"JSON_GROUP_ARRAY(JSON_OBJECT('id', HEX("tags"."id"), 'title', "tags"."title") ORDER BY "tags"."title") FILTER (WHERE "tags"."id" IS NOT NULL)"#,
            );
        
            let tags_subquery = params.tags.map(|e| {
                Expr::cust_with_expr(
                    r#"EXISTS (SELECT 1 FROM JSON_EACH("json_tags"."tags") AS "t" WHERE ?)"#,
                    Expr::cust(r#""t"."value" ->> 'title'"#).is_in(e),
                )
            });
            
            let (sql, values) = crate::user_feed::select(
                params.id,
                params.user_id,
                params.pinned,
                params.cursor,
                params.limit,
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
        })
        .await
        .unwrap()
        .map_err(|e: rusqlite::Error| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for SqliteFeedRepository {
    type Data = FeedCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let url = data.url.clone();

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            let (sql, values) =
                crate::feed::select_by_url(data.url).build_rusqlite(SqliteQueryBuilder);

            let feed_id = tx
                .prepare_cached(&sql)?
                .query_row(&*values.as_params(), |row| row.get::<_, i32>("id"))?;

            let pf_id = {
                let (mut sql, mut values) =
                    crate::user_feed::select_by_unique_index(data.user_id, feed_id)
                        .build_rusqlite(SqliteQueryBuilder);

                if let Some(id) = tx
                    .prepare_cached(&sql)?
                    .query_row(&*values.as_params(), |row| row.get::<_, Uuid>("id"))
                    .optional()?
                {
                    id
                } else {
                    (sql, values) = crate::user_feed::insert(
                        Some(Uuid::new_v4()),
                        data.pinned,
                        feed_id,
                        data.user_id,
                    )
                    .build_rusqlite(SqliteQueryBuilder);

                    tx.prepare_cached(&sql)?
                        .query_row(&*values.as_params(), |row| row.get::<_, Uuid>("id"))?
                }
            };

            link_entries_to_users(&tx, feed_id)?;

            if let Some(tags) = data.tags {
                link_tags(&tx, pf_id, tags, data.user_id)?;
            }

            tx.commit()?;

            Ok(pf_id)
        })
        .await
        .unwrap()
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error::Conflict(url),
            _ => Error::Unknown(e.into()),
        })
    }
}

#[async_trait::async_trait]
impl Updatable for SqliteFeedRepository {
    type Params = IdParams;
    type Data = FeedUpdateData;
    type Output = Result<(), Error>;

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
                    let (sql, values) = crate::user_feed::update(
                        params.id,
                        params.user_id,
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
                link_tags(&tx, params.id, tags, params.user_id)?;
            }

            tx.commit()
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
            let count = {
                let (sql, values) = crate::user_feed::delete(params.id, params.user_id)
                    .build_rusqlite(SqliteQueryBuilder);

                conn.execute(&sql, &*values.as_params())?
            };
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

    async fn stream(&self) -> Result<BoxStream<String>, Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

         conn.interact(move |conn| {
            let (sql, values) = crate::feed::iterate().build_rusqlite(SqliteQueryBuilder);

            let mut stmt = conn.prepare_cached(&sql)?;
            let mut rows = stmt.query(&*values.as_params())?;

            let mut urls: Vec<String> = Vec::new();
            while let Some(row) = rows.next()? {
                urls.push(row.get::<_, String>("url")?);
            }

            Ok(stream::iter(urls).boxed())
        })
        .await
        .unwrap()
        .map_err(|e: rusqlite::Error| Error::Unknown(e.into()))
        
        
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
                Value::Text(text) => serde_json::from_str(&text).ok(),
                _ => None,
            })?,
            unread_count: Some(value.get("unread_count")?),
        }))
    }
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
            crate::feed::insert(link, feed.title, url).build_rusqlite(SqliteQueryBuilder);

        conn.prepare_cached(&sql)?
            .query_row(&*values.as_params(), |row| row.get::<_, i32>("id"))?
    };

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

    let (sql, values) =
        crate::feed_entry::insert_many(&insert_many, feed_id).build_rusqlite(SqliteQueryBuilder);

    conn.prepare_cached(&sql)?.execute(&*values.as_params())?;

    Ok(feed_id)
}

pub(crate) fn link_entries_to_users(conn: &Connection, feed_id: i32) -> rusqlite::Result<()> {
    let fe_ids = {
        let (sql, values) =
            crate::feed_entry::select_many_by_feed_id(feed_id).build_rusqlite(SqliteQueryBuilder);

        let mut ids: Vec<i32> = Vec::new();

        let mut stmt = conn.prepare_cached(&sql)?;
        let mut rows = stmt.query(&*values.as_params())?;

        while let Some(row) = rows.next()? {
            ids.push(row.get("id")?);
        }

        ids
    };

    if !fe_ids.is_empty() {
        let insert_many = fe_ids
            .into_iter()
            .map(|feed_entry_id| crate::user_feed_entry::InsertMany {
                id: Some(Uuid::new_v4()),
                feed_entry_id,
            })
            .collect::<Vec<_>>();

        let (sql, values) =
            crate::user_feed_entry::insert_many_for_all_users(&insert_many, feed_id)
                .build_rusqlite(SqliteQueryBuilder);

        conn.prepare_cached(&sql)?.execute(&*values.as_params())?;
    }

    Ok(())
}

pub(crate) fn link_tags(
    conn: &Connection,
    user_feed_id: Uuid,
    tags: Vec<String>,
    user_id: Uuid,
) -> rusqlite::Result<()> {
    let (sql, values) = crate::user_feed_tag::delete_many_not_in_titles(&tags, user_id)
        .build_rusqlite(SqliteQueryBuilder);

    conn.prepare_cached(&sql)?.execute(&*values.as_params())?;

    let tag_ids = {
        let (sql, values) = crate::tag::insert_many(
            &tags
                .iter()
                .map(|e| crate::tag::InsertMany {
                    id: Some(Uuid::new_v4()),
                    title: e.to_owned(),
                })
                .collect::<Vec<_>>(),
            user_id,
        )
        .build_rusqlite(SqliteQueryBuilder);

        conn.prepare_cached(&sql)?.execute(&*values.as_params())?;

        let (sql, values) =
            crate::tag::select_ids_by_titles(&tags, user_id).build_rusqlite(SqliteQueryBuilder);

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
        .map(|e| crate::user_feed_tag::InsertMany {
            user_feed_id,
            tag_id: e,
        })
        .collect::<Vec<_>>();

    let (sql, values) = crate::user_feed_tag::insert_many(&insert_many, user_id)
        .build_rusqlite(SqliteQueryBuilder);

    conn.prepare_cached(&sql)?.execute(&*values.as_params())?;

    Ok(())
}
