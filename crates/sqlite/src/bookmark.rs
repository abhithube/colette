use chrono::{DateTime, Utc};
use colette_core::{
    bookmark::{
        BookmarkCreateData, BookmarkFindManyFilters, BookmarkRepository, BookmarkUpdateData,
        Cursor, Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, TagsLinkAction, TagsLinkData, Updatable},
    Bookmark,
};
use deadpool_sqlite::Pool;
use rusqlite::{types::Value, Connection, OptionalExtension, Row};
use sea_query::{Expr, ExprTrait, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use uuid::Uuid;

pub struct SqliteBookmarkRepository {
    pub(crate) pool: Pool,
}

impl SqliteBookmarkRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteBookmarkRepository {
    type Params = IdParams;
    type Output = Result<Bookmark, Error>;

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
impl Creatable for SqliteBookmarkRepository {
    type Data = BookmarkCreateData;
    type Output = Result<Bookmark, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            let bookmark_id = {
                let (sql, values) = colette_sql::bookmark::insert(
                    data.url,
                    data.bookmark.title,
                    data.bookmark.thumbnail.map(String::from),
                    data.bookmark.published.map(DateTime::<Utc>::from),
                    data.bookmark.author,
                )
                .build_rusqlite(SqliteQueryBuilder);

                tx.prepare_cached(&sql)?
                    .query_row(&*values.as_params(), |row| row.get::<_, i32>("id"))?
            };

            let pb_id = {
                let (mut sql, mut values) = colette_sql::profile_bookmark::select_by_unique_index(
                    data.profile_id,
                    bookmark_id,
                )
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
                        colette_sql::profile_bookmark::insert(id, bookmark_id, data.profile_id)
                            .build_rusqlite(SqliteQueryBuilder);

                    tx.prepare_cached(&sql)?.execute(&*values.as_params())?;

                    id
                }
            };

            if let Some(tags) = data.tags {
                link_tags(&tx, pb_id, tags, data.profile_id)?;
            }

            let bookmark = find_by_id(&tx, pb_id, data.profile_id)?;

            tx.commit()?;

            Ok::<_, rusqlite::Error>(bookmark)
        })
        .await
        .unwrap()
        .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Updatable for SqliteBookmarkRepository {
    type Params = IdParams;
    type Data = BookmarkUpdateData;
    type Output = Result<Bookmark, Error>;

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

            let bookmark = find_by_id(&tx, params.id, params.profile_id)?;

            tx.commit()?;

            Ok(bookmark)
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
            let (sql, values) = colette_sql::profile_bookmark::delete(params.id, params.profile_id)
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
impl BookmarkRepository for SqliteBookmarkRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
        filters: Option<BookmarkFindManyFilters>,
    ) -> Result<Vec<Bookmark>, Error> {
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
                Value::Text(text) => Some(serde_json::from_str(&text).unwrap()),
                _ => Some(Vec::new()),
            })?,
        }))
    }
}

pub(crate) fn find(
    conn: &Connection,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<BookmarkFindManyFilters>,
) -> rusqlite::Result<Vec<Bookmark>> {
    let mut tags: Option<Vec<String>> = None;

    if let Some(filters) = filters {
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

    let (sql, values) = colette_sql::profile_bookmark::select(
        id,
        profile_id,
        cursor,
        limit,
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
}

fn find_by_id(conn: &Connection, id: Uuid, profile_id: Uuid) -> rusqlite::Result<Bookmark> {
    let mut bookmarks = find(conn, Some(id), profile_id, None, None, None)?;
    if bookmarks.is_empty() {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    Ok(bookmarks.swap_remove(0))
}

pub(crate) fn link_tags(
    conn: &Connection,
    profile_bookmark_id: Uuid,
    tags: TagsLinkData,
    profile_id: Uuid,
) -> rusqlite::Result<()> {
    if let TagsLinkAction::Remove = tags.action {
        let (sql, values) =
            colette_sql::profile_bookmark_tag::delete_many_in_titles(&tags.data, profile_id)
                .build_rusqlite(SqliteQueryBuilder);

        conn.prepare_cached(&sql)?.execute(&*values.as_params())?;

        return Ok(());
    }

    if let TagsLinkAction::Set = tags.action {
        let (sql, values) =
            colette_sql::profile_bookmark_tag::delete_many_not_in_titles(&tags.data, profile_id)
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
        .map(|e| colette_sql::profile_bookmark_tag::InsertMany {
            profile_bookmark_id,
            tag_id: e,
        })
        .collect::<Vec<_>>();

    {
        let (sql, values) = colette_sql::profile_bookmark_tag::insert_many(insert_many, profile_id)
            .build_rusqlite(SqliteQueryBuilder);

        conn.prepare_cached(&sql)?.execute(&*values.as_params())?;
    }

    Ok(())
}
