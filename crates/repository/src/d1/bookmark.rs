use std::sync::Arc;

use chrono::{DateTime, NaiveDateTime, Utc};
use colette_core::{
    bookmark::{
        BookmarkCacheData, BookmarkCreateData, BookmarkFindParams, BookmarkRepository,
        BookmarkUpdateData, Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    Bookmark,
};
use sea_query::{Expr, ExprTrait, SqliteQueryBuilder, WithQuery};
use uuid::Uuid;
use worker::D1Database;

use super::{D1Binder, D1Values};

#[derive(Clone)]
pub struct D1BookmarkRepository {
    db: Arc<D1Database>,
}

impl D1BookmarkRepository {
    pub fn new(db: Arc<D1Database>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for D1BookmarkRepository {
    type Params = BookmarkFindParams;
    type Output = Result<Vec<Bookmark>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let (sql, values) = build_select(params).build_d1(SqliteQueryBuilder);

        let result = super::all(&self.db, sql, values)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        result
            .results::<BookmarkSelect>()
            .map(|e| e.into_iter().map(Bookmark::from).collect())
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for D1BookmarkRepository {
    type Data = BookmarkCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let bookmark_id = {
            let (sql, values) =
                crate::bookmark::select_by_link(data.url.clone()).build_d1(SqliteQueryBuilder);

            let Some(id) = super::first::<Uuid>(&self.db, sql, values, Some("id"))
                .await
                .map_err(|e| Error::Unknown(e.into()))?
            else {
                return Err(Error::Conflict(data.url));
            };

            id
        };

        let pb_id = {
            let (mut sql, mut values) =
                crate::user_bookmark::select_by_unique_index(data.user_id, bookmark_id)
                    .build_d1(SqliteQueryBuilder);

            if let Some(id) = super::first::<Uuid>(&self.db, sql, values, Some("id"))
                .await
                .map_err(|e| Error::Unknown(e.into()))?
            {
                id
            } else {
                let id = Uuid::new_v4();

                (sql, values) = crate::user_bookmark::insert(
                    Some(id),
                    data.title,
                    data.thumbnail_url,
                    data.published_at,
                    data.author,
                    data.folder_id,
                    bookmark_id,
                    data.user_id,
                )
                .build_d1(SqliteQueryBuilder);

                super::run(&self.db, sql, values)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;

                id
            }
        };

        if let Some(tags) = data.tags {
            link_tags(&self.db, pb_id, tags, data.user_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        Ok(pb_id)
    }
}

#[async_trait::async_trait]
impl Updatable for D1BookmarkRepository {
    type Params = IdParams;
    type Data = BookmarkUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        if data.title.is_some()
            || data.thumbnail_url.is_some()
            || data.published_at.is_some()
            || data.author.is_some()
            || data.folder_id.is_some()
        {
            let (sql, values) = crate::user_bookmark::update(
                params.id,
                data.title,
                data.thumbnail_url,
                data.published_at,
                data.author,
                data.folder_id,
                params.user_id,
            )
            .build_d1(SqliteQueryBuilder);

            let result = super::run(&self.db, sql, values)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
            let meta = result.meta().unwrap().unwrap();

            if meta.changes.is_none_or(|e| e == 0) {
                return Err(Error::NotFound(params.id));
            }
        }

        if let Some(tags) = data.tags {
            link_tags(&self.db, params.id, tags, params.user_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for D1BookmarkRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let (sql, values) =
            crate::user_bookmark::delete(params.id, params.user_id).build_d1(SqliteQueryBuilder);

        let result = super::run(&self.db, sql, values)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        let meta = result.meta().unwrap().unwrap();

        if meta.changes.is_none_or(|e| e == 0) {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl BookmarkRepository for D1BookmarkRepository {
    async fn cache(&self, data: BookmarkCacheData) -> Result<(), Error> {
        let (sql, values) = crate::bookmark::insert(
            Some(Uuid::new_v4()),
            data.url,
            data.bookmark.title,
            data.bookmark.thumbnail.map(String::from),
            data.bookmark.published,
            data.bookmark.author,
        )
        .build_d1(SqliteQueryBuilder);

        super::run(&self.db, sql, values)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
    }
}

pub(crate) fn build_select(params: BookmarkFindParams) -> WithQuery {
    let jsonb_agg = Expr::cust(
        r#"JSON_GROUP_ARRAY(JSON_OBJECT('id', "tags"."id", 'title', "tags"."title") ORDER BY "tags"."title") FILTER (WHERE "tags"."id" IS NOT NULL)"#,
    );

    let tags_subquery = params.tags.map(|e| {
        Expr::cust_with_expr(
            r#"EXISTS (SELECT 1 FROM JSON_EACH("json_tags"."tags") AS "t" WHERE ?)"#,
            Expr::cust(r#""t"."value" ->> 'title'"#).is_in(e),
        )
    });

    crate::user_bookmark::select(
        params.id,
        params.folder_id,
        params.user_id,
        params.cursor,
        params.limit,
        jsonb_agg,
        tags_subquery,
    )
}

#[worker::send]
pub(crate) async fn link_tags(
    db: &D1Database,
    user_bookmark_id: Uuid,
    tags: Vec<String>,
    user_id: Uuid,
) -> worker::Result<()> {
    let insert_many = tags
        .iter()
        .map(|e| crate::tag::InsertMany {
            id: Some(Uuid::new_v4()),
            title: e.to_owned(),
        })
        .collect::<Vec<_>>();

    let queries: Vec<(String, D1Values)> = vec![
        crate::tag::insert_many(&insert_many, user_id).build_d1(SqliteQueryBuilder),
        crate::tag::select_ids_by_titles(&tags, user_id).build_d1(SqliteQueryBuilder),
    ];

    let results = super::batch(db, queries).await?;
    let tag_ids = {
        #[derive(serde::Deserialize)]
        struct Tag {
            id: Uuid,
        }

        results
            .last()
            .unwrap()
            .results::<Tag>()
            .unwrap()
            .into_iter()
            .map(|e| e.id)
            .collect::<Vec<_>>()
    };

    let insert_many = tag_ids
        .into_iter()
        .map(|e| crate::user_bookmark_tag::InsertMany {
            user_bookmark_id,
            tag_id: e,
        })
        .collect::<Vec<_>>();

    let queries: Vec<(String, D1Values)> = vec![
        crate::user_bookmark_tag::delete_many_not_in_titles(&tags, user_id)
            .build_d1(SqliteQueryBuilder),
        crate::user_bookmark_tag::insert_many(&insert_many, user_id).build_d1(SqliteQueryBuilder),
    ];

    super::batch(db, queries).await?;

    Ok(())
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct BookmarkSelect {
    pub id: Uuid,
    pub link: String,
    pub title: Option<String>,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub original_title: String,
    pub original_thumbnail_url: Option<String>,
    pub original_published_at: Option<DateTime<Utc>>,
    pub original_author: Option<String>,
    pub folder_id: Option<Uuid>,
    pub created_at: String,
    pub tags: Option<String>,
}

impl From<BookmarkSelect> for Bookmark {
    fn from(value: BookmarkSelect) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author,
            original_title: value.original_title,
            original_thumbnail_url: value.original_thumbnail_url,
            original_published_at: value.original_published_at,
            original_author: value.original_author,
            folder_id: value.folder_id,
            created_at: NaiveDateTime::parse_from_str(&value.created_at, "%Y-%m-%d %H:%M:%S")
                .unwrap()
                .and_utc(),
            tags: value
                .tags
                .as_ref()
                .and_then(|e| serde_json::de::from_str(e).ok()),
        }
    }
}
