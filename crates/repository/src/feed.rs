use chrono::NaiveDateTime;
use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    feed::{
        ConflictError, Error, FeedCacheData, FeedCreateData, FeedFindParams, FeedRepository,
        FeedUpdateData, ProcessedFeed,
    },
    Feed,
};
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use sqlx::{PgConnection, Pool, Postgres};
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
        crate::common::select_feeds(
            &self.pool,
            params.id,
            params.folder_id,
            params.user_id,
            params.cursor,
            params.limit,
            params.tags,
        )
        .await
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

        let feed_id = sqlx::query_file_scalar!("queries/feeds/select_by_url.sql", data.url)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => {
                    Error::Conflict(ConflictError::NotCached(data.url.clone()))
                }
                _ => Error::Unknown(e.into()),
            })?;

        let uf_id = {
            if let Some(id) = sqlx::query_file_scalar!(
                "queries/user_feeds/select_by_index.sql",
                data.user_id,
                feed_id
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| Error::Unknown(e.into()))?
            {
                id
            } else {
                sqlx::query_file_scalar!(
                    "queries/user_feeds/insert.sql",
                    data.title,
                    data.folder_id,
                    feed_id,
                    data.user_id
                )
                .fetch_one(&mut *tx)
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
            sqlx::query_file_scalar!(
                "queries/user_feed_tags/link.sql",
                uf_id,
                data.user_id,
                &tags
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(uf_id)
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

        if data.title.is_some() || data.folder_id.is_some() {
            let (has_title, title) = match data.title {
                Some(title) => (true, title),
                None => (false, None),
            };
            let (has_folder, folder_id) = match data.folder_id {
                Some(folder_id) => (true, folder_id),
                None => (false, None),
            };

            sqlx::query_file!(
                "queries/user_feeds/update.sql",
                params.id,
                params.user_id,
                has_title,
                title,
                has_folder,
                folder_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;
        }

        if let Some(tags) = data.tags {
            sqlx::query_file_scalar!(
                "queries/user_feed_tags/link.sql",
                params.id,
                params.user_id,
                &tags
            )
            .execute(&mut *tx)
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
        sqlx::query_file!("queries/user_feeds/delete.sql", params.id, params.user_id)
            .execute(&self.pool)
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
        sqlx::query_file_scalar!("queries/feeds/stream.sql")
            .fetch(&self.pool)
            .map_err(|e| Error::Unknown(e.into()))
            .boxed()
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

        sqlx::query_file_scalar!("queries/feeds/insert.sql", link, feed.title, xml_url)
            .fetch_one(&mut *conn)
            .await?
    };

    if !feed.entries.is_empty() {
        let mut links = Vec::<String>::new();
        let mut titles = Vec::<String>::new();
        let mut published_ats = Vec::<NaiveDateTime>::new();
        let mut descriptions = Vec::<Option<String>>::new();
        let mut authors = Vec::<Option<String>>::new();
        let mut thumbnail_urls = Vec::<Option<String>>::new();

        for item in feed.entries {
            links.push(item.link.to_string());
            titles.push(item.title);
            published_ats.push(item.published.naive_utc());
            descriptions.push(item.description);
            authors.push(item.author);
            thumbnail_urls.push(item.thumbnail.map(String::from));
        }

        sqlx::query_file_scalar!(
            "queries/feed_entries/insert_many.sql",
            &links,
            &titles,
            &published_ats,
            &descriptions as &[Option<String>],
            &authors as &[Option<String>],
            &thumbnail_urls as &[Option<String>],
            feed_id
        )
        .execute(&mut *conn)
        .await?;
    }

    Ok(feed_id)
}

pub(crate) async fn link_entries_to_users(
    conn: &mut PgConnection,
    feed_id: Uuid,
) -> Result<(), sqlx::Error> {
    let fe_ids = sqlx::query_file_scalar!("queries/feed_entries/select_by_feed.sql", feed_id)
        .fetch_all(&mut *conn)
        .await?;

    if !fe_ids.is_empty() {
        sqlx::query_file!(
            "queries/user_feed_entries/insert_many.sql",
            &fe_ids,
            feed_id
        )
        .execute(&mut *conn)
        .await?;
    }

    Ok(())
}
