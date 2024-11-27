use colette_core::{
    bookmark::{
        BookmarkCacheData, BookmarkCreateData, BookmarkFindParams, BookmarkRepository,
        BookmarkUpdateData, Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    Bookmark,
};
use deadpool_sqlite::{
    rusqlite::{self, types::Value, Connection, OptionalExtension, Row},
    Pool,
};
use sea_query::{Expr, ExprTrait, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteBookmarkRepository {
    pool: Pool,
}

impl SqliteBookmarkRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteBookmarkRepository {
    type Params = BookmarkFindParams;
    type Output = Result<Vec<Bookmark>, Error>;

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
            
            let (sql, values) = crate::profile_bookmark::select(
                params.id,
                params.profile_id,
                params.cursor,
                params.limit,
                jsonb_agg,
                tags_subquery,
            )
            .build_rusqlite(SqliteQueryBuilder);

            let mut stmt = conn.prepare_cached(&sql)?;
            let mut rows = stmt.query(&*values.as_params())?;

            let mut bookmarks: Vec<Bookmark> = Vec::new();
            while let Some(row) = rows.next()? {
                bookmarks.push(BookmarkSelect::try_from(row).map(|e| e.0)?);
            }

            Ok(bookmarks)
        })
        .await
        .unwrap()
        .map_err(|e: rusqlite::Error| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for SqliteBookmarkRepository {
    type Data = BookmarkCreateData;
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

            let (sql, values) = crate::bookmark::select_by_link(data.url.clone())
                .build_rusqlite(SqliteQueryBuilder);

            let bookmark_id = tx
                .prepare_cached(&sql)?
                .query_row(&*values.as_params(), |row| row.get::<_, i32>("id"))?;

            let pb_id = {
                let (mut sql, mut values) =
                    crate::profile_bookmark::select_by_unique_index(data.profile_id, bookmark_id)
                        .build_rusqlite(SqliteQueryBuilder);

                if let Some(id) = tx
                    .prepare_cached(&sql)?
                    .query_row(&*values.as_params(), |row| row.get::<_, Uuid>("id"))
                    .optional()?
                {
                    id
                } else {
                    (sql, values) = crate::profile_bookmark::insert(
                        Some(Uuid::new_v4()),
                        bookmark_id,
                        data.profile_id,
                    )
                    .build_rusqlite(SqliteQueryBuilder);

                    tx.prepare_cached(&sql)?
                        .query_row(&*values.as_params(), |row| row.get::<_, Uuid>("id"))?
                }
            };

            if let Some(tags) = data.tags {
                link_tags(&tx, pb_id, tags, data.profile_id)?;
            }

            tx.commit()?;

            Ok(pb_id)
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
impl Updatable for SqliteBookmarkRepository {
    type Params = IdParams;
    type Data = BookmarkUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            if let Some(tags) = data.tags {
                link_tags(&tx, params.id, tags, params.profile_id)?;
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
impl Deletable for SqliteBookmarkRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let (sql, values) = crate::profile_bookmark::delete(params.id, params.profile_id)
                .build_rusqlite(SqliteQueryBuilder);

            let count = conn.prepare_cached(&sql)?.execute(&*values.as_params())?;
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
impl BookmarkRepository for SqliteBookmarkRepository {
    async fn cache(&self, data: BookmarkCacheData) -> Result<(), Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let (sql, values) = crate::bookmark::insert(
                data.url,
                data.bookmark.title,
                data.bookmark.thumbnail.map(String::from),
                data.bookmark.published,
                data.bookmark.author,
            )
            .build_rusqlite(SqliteQueryBuilder);

            conn.prepare_cached(&sql)?.execute(&*values.as_params())?;

            Ok(())
        })
        .await
        .unwrap()
        .map_err(|e: rusqlite::Error| Error::Unknown(e.into()))
    }
}

#[derive(Debug, Clone)]
struct BookmarkSelect(Bookmark);

impl TryFrom<&Row<'_>> for BookmarkSelect {
    type Error = rusqlite::Error;

    fn try_from(value: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self(Bookmark {
            id: value.get("id")?,
            link: value.get("link")?,
            title: value.get("title")?,
            thumbnail_url: value.get("thumbnail_url")?,
            published_at: value.get("published_at")?,
            author: value.get("author")?,
            created_at: value.get("created_at")?,
            tags: value.get::<_, Value>("tags").map(|e| match e {
                Value::Text(text) => serde_json::from_str(&text).ok(),
                _ => None,
            })?,
        }))
    }
}

pub(crate) fn link_tags(
    conn: &Connection,
    profile_bookmark_id: Uuid,
    tags: Vec<String>,
    profile_id: Uuid,
) -> rusqlite::Result<()> {
    let (sql, values) = crate::profile_bookmark_tag::delete_many_not_in_titles(&tags, profile_id)
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
            profile_id,
        )
        .build_rusqlite(SqliteQueryBuilder);

        conn.prepare_cached(&sql)?.execute(&*values.as_params())?;

        let (sql, values) =
            crate::tag::select_ids_by_titles(&tags, profile_id).build_rusqlite(SqliteQueryBuilder);

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
        .map(|e| crate::profile_bookmark_tag::InsertMany {
            profile_bookmark_id,
            tag_id: e,
        })
        .collect::<Vec<_>>();

    let (sql, values) = crate::profile_bookmark_tag::insert_many(&insert_many, profile_id)
        .build_rusqlite(SqliteQueryBuilder);

    conn.prepare_cached(&sql)?.execute(&*values.as_params())?;

    Ok(())
}
