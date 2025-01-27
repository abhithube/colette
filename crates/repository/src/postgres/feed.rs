use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    feed::{
        ConflictError, Error, FeedCacheData, FeedCreateData, FeedFindParams, FeedRepository,
        FeedUpdateData, ProcessedFeed,
    },
    Feed,
};
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use sqlx::{postgres::PgRow, types::Json, PgConnection, Pool, Postgres, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresFeedRepository {
    pool: Pool<Postgres>,
}

impl PostgresFeedRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresFeedRepository {
    type Params = FeedFindParams;
    type Output = Result<Vec<Feed>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        crate::user_feed::select(
            &self.pool,
            params.id,
            params.folder_id,
            params.user_id,
            params.cursor,
            params.limit,
            params.tags,
        )
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
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let feed_id = crate::feed::select_by_url(&mut *tx, data.url.clone())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => {
                    Error::Conflict(ConflictError::NotCached(data.url.clone()))
                }
                _ => Error::Unknown(e.into()),
            })?;

        let pf_id = {
            if let Some(id) =
                crate::user_feed::select_by_unique_index(&mut *tx, data.user_id, feed_id)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
            {
                id
            } else {
                crate::user_feed::insert(
                    &mut *tx,
                    data.title,
                    data.folder_id,
                    feed_id,
                    data.user_id,
                )
                .await
                .map_err(|e| match e {
                    sqlx::Error::Database(e) if e.is_unique_violation() => {
                        Error::Conflict(ConflictError::AlreadyExists(data.url))
                    }
                    _ => Error::Unknown(e.into()),
                })?
            }
        };

        link_entries_to_users(&mut tx, feed_id)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if let Some(tags) = data.tags {
            link_tags(&mut tx, pf_id, &tags, data.user_id)
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
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.title.is_some() {
            crate::user_feed::update(
                &mut *tx,
                params.id,
                params.user_id,
                data.title,
                data.folder_id,
            )
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;
        }

        if let Some(tags) = data.tags {
            link_tags(&mut tx, params.id, &tags, params.user_id)
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
        crate::user_feed::delete(&self.pool, params.id, params.user_id)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl FeedRepository for PostgresFeedRepository {
    async fn cache(&self, data: FeedCacheData) -> Result<(), Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        create_feed_with_entries(&mut tx, data.url, data.feed)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }

    fn stream(&self) -> BoxStream<Result<String, Error>> {
        sqlx::query_scalar::<_, String>("SELECT coalesce(xml_url, link) AS url FROM feeds JOIN user_feeds ON user_feeds.feed_id = feeds.id")
            .fetch(&self.pool)
            .map_err(|e| Error::Unknown(e.into()))
            .boxed()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FeedSelect(pub(crate) Feed);

impl From<PgRow> for FeedSelect {
    fn from(value: PgRow) -> Self {
        Self(Feed {
            id: value.get("id"),
            link: value.get("link"),
            title: value.get("title"),
            xml_url: value.get("xml_url"),
            original_title: value.get("original_title"),
            folder_id: value.get("folder_id"),
            tags: value
                .get::<Option<Json<Vec<colette_core::Tag>>>, _>("tags")
                .map(|e| e.0),
            unread_count: Some(value.get("unread_count")),
        })
    }
}

pub(crate) async fn create_feed_with_entries(
    conn: &mut PgConnection,
    url: String,
    feed: ProcessedFeed,
) -> Result<Uuid, sqlx::Error> {
    let feed_id = {
        let link = feed.link.to_string();
        let xml_url = if url == link { None } else { Some(url) };

        crate::feed::insert(&mut *conn, link, feed.title, xml_url).await?
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

        crate::feed_entry::insert_many(&mut *conn, insert_many, feed_id).await?;
    }

    Ok(feed_id)
}

pub(crate) async fn link_entries_to_users(
    conn: &mut PgConnection,
    feed_id: Uuid,
) -> Result<(), sqlx::Error> {
    let fe_ids = { crate::feed_entry::select_many_by_feed_id(&mut *conn, feed_id).await? };

    if !fe_ids.is_empty() {
        // let insert_many = fe_ids
        //     .into_iter()
        //     .map(|feed_entry_id| crate::user_feed_entry::InsertMany {
        //         id: None,
        //         feed_entry_id,
        //     })
        //     .collect::<Vec<_>>();

        crate::user_feed_entry::insert_many(&mut *conn, &fe_ids, feed_id).await?;
    }

    Ok(())
}

pub(crate) async fn link_tags(
    conn: &mut PgConnection,
    user_feed_id: Uuid,
    tags: &[String],
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    crate::user_feed_tag::delete_many(&mut *conn, tags, user_id).await?;

    crate::tag::insert_many(&mut *conn, tags, user_id).await?;

    crate::user_feed_tag::insert_many(&mut *conn, user_feed_id, tags, user_id).await?;

    Ok(())
}
