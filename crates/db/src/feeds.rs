use colette_core::{
    common::FindOneParams,
    feeds::{
        Error, FeedsCreateData, FeedsFindManyParams, FeedsRepository, FeedsUpdateData, StreamFeed,
    },
};
use futures::{stream::BoxStream, StreamExt};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{common::convert_chrono_to_time, tags::Tag, PostgresRepository};

#[async_trait::async_trait]
impl FeedsRepository for PostgresRepository {
    async fn find_many_feeds(
        &self,
        params: FeedsFindManyParams,
    ) -> Result<Vec<colette_core::Feed>, Error> {
        sqlx::query_file_as!(
            Feed,
            "queries/feeds/find_many.sql",
            params.profile_id,
            params.tags.as_deref()
        )
        .fetch_all(&self.pool)
        .await
        .map(|e| e.into_iter().map(colette_core::Feed::from).collect())
        .map_err(|e| Error::Unknown(e.into()))
    }

    async fn find_one_feed(&self, params: FindOneParams) -> Result<colette_core::Feed, Error> {
        sqlx::query_file_as!(
            Feed,
            "queries/feeds/find_one.sql",
            params.id,
            params.profile_id
        )
        .fetch_one(&self.pool)
        .await
        .map(colette_core::Feed::from)
        .map_err(|e| Error::Unknown(e.into()))
    }

    async fn create_feed(&self, data: FeedsCreateData) -> Result<colette_core::Feed, Error> {
        let links = data
            .feed
            .entries
            .iter()
            .map(|entry| entry.link.as_str())
            .collect::<Vec<_>>();
        let titles = data
            .feed
            .entries
            .iter()
            .map(|entry| entry.title.as_str())
            .collect::<Vec<_>>();
        let published_dates = data
            .feed
            .entries
            .iter()
            .map(|entry| entry.published.map(convert_chrono_to_time))
            .collect::<Vec<_>>();
        let descriptions = data
            .feed
            .entries
            .iter()
            .map(|entry| entry.description.as_deref())
            .collect::<Vec<_>>();
        let authors = data
            .feed
            .entries
            .iter()
            .map(|entry| entry.author.as_deref())
            .collect::<Vec<_>>();
        let thumbnails = data
            .feed
            .entries
            .iter()
            .map(|entry| entry.thumbnail.as_ref().map(|e| e.as_str()))
            .collect::<Vec<_>>();

        let link = data.feed.link.as_str();
        let url = if data.url == link {
            None
        } else {
            Some(&data.url)
        };
        sqlx::query_file_as!(
            Feed,
            "queries/feeds/insert.sql",
            data.profile_id,
            link,
            data.feed.title,
            url,
            &links as &[&str],
            &titles as &[&str],
            &published_dates as &[Option<OffsetDateTime>],
            &descriptions as &[Option<&str>],
            &authors as &[Option<&str>],
            &thumbnails as &[Option<&str>]
        )
        .fetch_one(&self.pool)
        .await
        .map(colette_core::Feed::from)
        .map_err(|e| Error::Unknown(e.into()))
    }

    async fn update_feed(
        &self,
        params: FindOneParams,
        data: FeedsUpdateData,
    ) -> Result<colette_core::Feed, Error> {
        match data.tags {
            Some(tags) => sqlx::query_file_as!(
                Feed,
                "queries/feeds/update.sql",
                params.id,
                params.profile_id,
                &tags
            )
            .fetch_one(&self.pool)
            .await
            .map(colette_core::Feed::from)
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            }),
            None => self.find_one_feed(params).await,
        }
    }

    async fn delete_feed(&self, params: FindOneParams) -> Result<(), Error> {
        let result = sqlx::query_file!("queries/feeds/delete.sql", params.id, params.profile_id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }

    fn stream_feeds(&self) -> BoxStream<Result<StreamFeed, Error>> {
        Box::pin(
            sqlx::query_file_as!(StreamFeed, "queries/feeds/stream.sql")
                .fetch(&self.pool)
                .map(|e| e.map_err(|e| Error::Unknown(e.into()))),
        )
    }

    async fn cleanup_feeds(&self) -> Result<(), Error> {
        sqlx::query_file!("queries/feeds/cleanup.sql")
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Feed {
    id: Uuid,
    link: String,
    title: String,
    url: Option<String>,
    tags: Vec<Tag>,
    unread_count: Option<i64>,
}

impl From<Feed> for colette_core::Feed {
    fn from(value: Feed) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            url: value.url,
            tags: value
                .tags
                .into_iter()
                .map(colette_core::Tag::from)
                .collect(),
            unread_count: value.unread_count,
        }
    }
}
