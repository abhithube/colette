use std::sync::Arc;

use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    feed::{
        Error, FeedCacheData, FeedCreateData, FeedFindParams, FeedRepository, FeedUpdateData,
        ProcessedFeed,
    },
    Feed,
};
use futures::{
    stream::{self, BoxStream},
    StreamExt,
};
use sea_query::{Expr, ExprTrait, SqliteQueryBuilder, WithQuery};
use uuid::Uuid;
use worker::D1Database;

use super::{D1Binder, D1Values};

#[derive(Clone)]
pub struct D1FeedRepository {
    db: Arc<D1Database>,
}

impl D1FeedRepository {
    pub fn new(db: Arc<D1Database>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for D1FeedRepository {
    type Params = FeedFindParams;
    type Output = Result<Vec<Feed>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let (sql, values) = build_select(params).build_d1(SqliteQueryBuilder);

        let result = super::all(&self.db, sql, values)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        result
            .results::<FeedSelect>()
            .map(|e| e.into_iter().map(Feed::from).collect())
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for D1FeedRepository {
    type Data = FeedCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let feed_id = {
            let (sql, values) =
                crate::feed::select_by_url(data.url.clone()).build_d1(SqliteQueryBuilder);

            let Some(id) = super::first::<Uuid>(&self.db, sql, values, Some("id"))
                .await
                .map_err(|e| Error::Unknown(e.into()))?
            else {
                return Err(Error::Conflict(data.url));
            };

            id
        };

        let pf_id = {
            let (mut sql, mut values) =
                crate::user_feed::select_by_unique_index(data.user_id, feed_id)
                    .build_d1(SqliteQueryBuilder);

            if let Some(id) = super::first::<Uuid>(&self.db, sql, values, Some("id"))
                .await
                .map_err(|e| Error::Unknown(e.into()))?
            {
                id
            } else {
                let id = Uuid::new_v4();

                (sql, values) = crate::user_feed::insert(
                    Some(id),
                    data.title,
                    data.folder_id,
                    feed_id,
                    data.user_id,
                )
                .build_d1(SqliteQueryBuilder);

                super::run(&self.db, sql, values)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;

                id
            }
        };

        link_entries_to_users(&self.db, feed_id)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if let Some(tags) = data.tags {
            link_tags(&self.db, pf_id, tags, data.user_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        Ok(pf_id)
    }
}

#[async_trait::async_trait]
impl Updatable for D1FeedRepository {
    type Params = IdParams;
    type Data = FeedUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        if data.title.is_some() {
            let (sql, values) =
                crate::user_feed::update(params.id, params.user_id, data.title, data.folder_id)
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
impl Deletable for D1FeedRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let (sql, values) =
            crate::user_feed::delete(params.id, params.user_id).build_d1(SqliteQueryBuilder);

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
impl FeedRepository for D1FeedRepository {
    async fn cache(&self, data: FeedCacheData) -> Result<(), Error> {
        create_feed_with_entries(&self.db, data.url, data.feed)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
    }

    async fn stream(&self) -> Result<BoxStream<String>, Error> {
        let (sql, values) = crate::feed::iterate().build_d1(SqliteQueryBuilder);

        let rows = super::all(&self.db, sql, values)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let urls = {
            #[derive(serde::Deserialize)]
            struct Feed {
                url: String,
            }

            rows.results::<Feed>()
                .unwrap()
                .into_iter()
                .map(|e| e.url)
                .collect::<Vec<_>>()
        };

        Ok(stream::iter(urls).boxed())
    }
}

pub(crate) fn build_select(params: FeedFindParams) -> WithQuery {
    let jsonb_agg = Expr::cust(
        r#"JSON_GROUP_ARRAY(JSON_OBJECT('id', "tags"."id", 'title', "tags"."title") ORDER BY "tags"."title") FILTER (WHERE "tags"."id" IS NOT NULL)"#,
    );

    let tags_subquery = params.tags.map(|e| {
        Expr::cust_with_expr(
            r#"EXISTS (SELECT 1 FROM JSON_EACH("json_tags"."tags") AS "t" WHERE ?)"#,
            Expr::cust(r#""t"."value" ->> 'title'"#).is_in(e),
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

const CHUNK_SIZE: usize = 14;

pub(crate) async fn create_feed_with_entries(
    db: &D1Database,
    url: String,
    feed: ProcessedFeed,
) -> worker::Result<Uuid> {
    let feed_id = {
        let link = feed.link.to_string();
        let xml_url = if url == link { None } else { Some(url) };

        let (sql, values) =
            crate::feed::insert(link, feed.title, xml_url).build_d1(SqliteQueryBuilder);

        super::first::<Uuid>(db, sql, values, Some("id"))
            .await?
            .unwrap()
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

        for chunk in insert_many.chunks(CHUNK_SIZE) {
            let (sql, values) =
                crate::feed_entry::insert_many(chunk, feed_id).build_d1(SqliteQueryBuilder);

            super::run(db, sql, values).await?;
        }
    }

    Ok(feed_id)
}

pub(crate) async fn link_entries_to_users(db: &D1Database, feed_id: Uuid) -> worker::Result<()> {
    let fe_ids = {
        let (sql, values) =
            crate::feed_entry::select_many_by_feed_id(feed_id).build_d1(SqliteQueryBuilder);

        let result = super::all(db, sql, values).await?;

        #[derive(serde::Deserialize)]
        struct FeedEntryId {
            id: Uuid,
        }

        result
            .results::<FeedEntryId>()
            .unwrap()
            .into_iter()
            .map(|e| e.id)
            .collect::<Vec<_>>()
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
                .build_d1(SqliteQueryBuilder);

        super::run(db, sql, values).await?;
    }

    Ok(())
}

#[worker::send]
pub(crate) async fn link_tags(
    db: &D1Database,
    user_feed_id: Uuid,
    tags: Vec<String>,
    user_id: Uuid,
) -> worker::Result<()> {
    let tag_ids = {
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

        #[derive(serde::Deserialize)]
        struct TagId {
            id: Uuid,
        }

        results
            .last()
            .unwrap()
            .results::<TagId>()
            .unwrap()
            .into_iter()
            .map(|e| e.id)
            .collect::<Vec<_>>()
    };

    let insert_many = tag_ids
        .into_iter()
        .map(|e| crate::user_feed_tag::InsertMany {
            user_feed_id,
            tag_id: e,
        })
        .collect::<Vec<_>>();

    let queries: Vec<(String, D1Values)> = vec![
        crate::user_feed_tag::delete_many_not_in_titles(&tags, user_id)
            .build_d1(SqliteQueryBuilder),
        crate::user_feed_tag::insert_many(&insert_many, user_id).build_d1(SqliteQueryBuilder),
    ];

    super::batch(db, queries).await?;

    Ok(())
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct FeedSelect {
    pub id: Uuid,
    pub link: String,
    pub title: Option<String>,
    pub original_title: String,
    pub xml_url: Option<String>,
    pub folder_id: Option<Uuid>,
    pub tags: Option<String>,
    pub unread_count: i64,
}

impl From<FeedSelect> for Feed {
    fn from(value: FeedSelect) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            original_title: value.original_title,
            xml_url: value.xml_url,
            folder_id: value.folder_id,
            tags: value
                .tags
                .as_ref()
                .and_then(|e| serde_json::de::from_str(e).ok()),
            unread_count: Some(value.unread_count),
        }
    }
}
