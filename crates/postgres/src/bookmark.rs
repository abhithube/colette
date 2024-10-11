use chrono::{DateTime, Utc};
use colette_core::{
    bookmark::{
        BookmarkCreateData, BookmarkFindManyFilters, BookmarkRepository, BookmarkUpdateData,
        Cursor, Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, TagsLinkAction, TagsLinkData, Updatable},
    Bookmark,
};
use deadpool_postgres::{GenericClient, Pool};
use sea_query::{Expr, ExprTrait, PostgresQueryBuilder};
use sea_query_postgres::PostgresBinder;
use tokio_postgres::{types::Json, Row};
use uuid::Uuid;

pub struct PostgresBookmarkRepository {
    pub(crate) pool: Pool,
}

impl PostgresBookmarkRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresBookmarkRepository {
    type Params = IdParams;
    type Output = Result<Bookmark, Error>;

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
impl Creatable for PostgresBookmarkRepository {
    type Data = BookmarkCreateData;
    type Output = Result<Bookmark, Error>;

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

        let bookmark_id = {
            let (sql, values) = colette_sql::bookmark::insert(
                data.url,
                data.bookmark.title,
                data.bookmark.thumbnail.map(String::from),
                data.bookmark.published.map(DateTime::<Utc>::from),
                data.bookmark.author,
            )
            .build_postgres(PostgresQueryBuilder);

            let row = tx
                .query_one(&sql, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            row.get("id")
        };

        let pb_id = {
            let (mut sql, mut values) =
                colette_sql::profile_bookmark::select_by_unique_index(data.profile_id, bookmark_id)
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
                    colette_sql::profile_bookmark::insert(id, bookmark_id, data.profile_id)
                        .build_postgres(PostgresQueryBuilder);

                tx.execute(&sql, &values.as_params())
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;

                id
            }
        };

        if let Some(tags) = data.tags {
            link_tags(&tx, pb_id, tags, data.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        let bookmark = find_by_id(&tx, IdParams::new(pb_id, data.profile_id))
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(bookmark)
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresBookmarkRepository {
    type Params = IdParams;
    type Data = BookmarkUpdateData;
    type Output = Result<Bookmark, Error>;

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

        if let Some(tags) = data.tags {
            link_tags(&tx, params.id, tags, params.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        let bookmark = find_by_id(&tx, params).await?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(bookmark)
    }
}

#[async_trait::async_trait]
impl Deletable for PostgresBookmarkRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let count = {
            let (sql, values) = colette_sql::profile_bookmark::delete(params.id, params.profile_id)
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
impl BookmarkRepository for PostgresBookmarkRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
        filters: Option<BookmarkFindManyFilters>,
    ) -> Result<Vec<Bookmark>, Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        find(&client, None, profile_id, limit, cursor, filters).await
    }
}

#[derive(Debug, Clone)]
struct BookmarkSelect(Bookmark);

impl From<&Row> for BookmarkSelect {
    fn from(value: &Row) -> Self {
        Self(Bookmark {
            id: value.get("id"),
            link: value.get("link"),
            title: value.get("title"),
            thumbnail_url: value.get("thumbnail_url"),
            published_at: value.get("published_at"),
            author: value.get("author"),
            created_at: value.get("created_at"),
            tags: value
                .get::<_, Option<Json<Vec<colette_core::Tag>>>>("tags")
                .map(|e| e.0),
        })
    }
}

pub(crate) async fn find<C: GenericClient>(
    client: &C,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<BookmarkFindManyFilters>,
) -> Result<Vec<Bookmark>, Error> {
    let mut tags: Option<Vec<String>> = None;

    if let Some(filters) = filters {
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

    let (sql, values) = colette_sql::profile_bookmark::select(
        id,
        profile_id,
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
                .map(|e| BookmarkSelect::from(&e).0)
                .collect::<Vec<_>>()
        })
        .map_err(|e| Error::Unknown(e.into()))
}

pub async fn find_by_id<C: GenericClient>(client: &C, params: IdParams) -> Result<Bookmark, Error> {
    let mut bookmarks = find(client, Some(params.id), params.profile_id, None, None, None).await?;
    if bookmarks.is_empty() {
        return Err(Error::NotFound(params.id));
    }

    Ok(bookmarks.swap_remove(0))
}

pub(crate) async fn link_tags<C: GenericClient>(
    client: &C,
    profile_bookmark_id: Uuid,
    tags: TagsLinkData,
    profile_id: Uuid,
) -> Result<(), tokio_postgres::Error> {
    if let TagsLinkAction::Remove = tags.action {
        let (sql, values) =
            colette_sql::profile_bookmark_tag::delete_many_in_titles(&tags.data, profile_id)
                .build_postgres(PostgresQueryBuilder);

        client.execute(&sql, &values.as_params()).await?;

        return Ok(());
    }

    if let TagsLinkAction::Set = tags.action {
        let (sql, values) =
            colette_sql::profile_bookmark_tag::delete_many_not_in_titles(&tags.data, profile_id)
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
        .map(|e| colette_sql::profile_bookmark_tag::InsertMany {
            profile_bookmark_id,
            tag_id: e,
        })
        .collect::<Vec<_>>();

    {
        let (sql, values) = colette_sql::profile_bookmark_tag::insert_many(insert_many, profile_id)
            .build_postgres(PostgresQueryBuilder);

        client.execute(&sql, &values.as_params()).await?;
    }

    Ok(())
}
