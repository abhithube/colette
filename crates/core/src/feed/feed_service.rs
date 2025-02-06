use std::sync::Arc;

use apalis_redis::{RedisContext, RedisError};
use chrono::{DateTime, Utc};
use futures::stream::BoxStream;
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;

use super::{
    DetectedFeed, DetectorResponse, Error, Feed, FeedDetector,
    feed_repository::{
        FeedCacheData, FeedCreateData, FeedFindParams, FeedRepository, FeedUpdateData,
    },
    feed_scraper::ProcessedFeed,
};
use crate::{
    common::{IdParams, NonEmptyString, Paginated},
    storage::Storage,
};

pub struct FeedService {
    repository: Box<dyn FeedRepository>,
    detector: Box<dyn FeedDetector>,
}

impl FeedService {
    pub fn new(repository: impl FeedRepository, detector: Box<dyn FeedDetector>) -> Self {
        Self {
            repository: Box::new(repository),
            detector,
        }
    }

    pub async fn list_feeds(
        &self,
        query: FeedListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<Feed>, Error> {
        let feeds = self
            .repository
            .find(FeedFindParams {
                tags: query.tags,
                user_id,
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: feeds,
            ..Default::default()
        })
    }

    pub async fn get_feed(&self, id: Uuid, user_id: Uuid) -> Result<Feed, Error> {
        let mut feeds = self
            .repository
            .find(FeedFindParams {
                id: Some(id),
                user_id,
                ..Default::default()
            })
            .await?;
        if feeds.is_empty() {
            return Err(Error::NotFound(id));
        }

        Ok(feeds.swap_remove(0))
    }

    pub async fn create_feed(&self, data: FeedCreate, user_id: Uuid) -> Result<Feed, Error> {
        let id = self
            .repository
            .create(FeedCreateData {
                url: data.url.to_string(),
                title: data.title.into(),
                folder_id: data.folder_id,
                tags: data.tags,
                user_id,
            })
            .await?;

        self.get_feed(id, user_id).await
    }

    pub async fn update_feed(
        &self,
        id: Uuid,
        data: FeedUpdate,
        user_id: Uuid,
    ) -> Result<Feed, Error> {
        self.repository
            .update(IdParams::new(id, user_id), data.into())
            .await?;

        self.get_feed(id, user_id).await
    }

    pub async fn delete_feed(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        self.repository.delete(IdParams::new(id, user_id)).await
    }

    pub async fn detect_feeds(&self, data: FeedDetect) -> Result<DetectedResponse, Error> {
        match self.detector.detect(data.url.clone()).await? {
            DetectorResponse::Detected(feeds) => Ok(DetectedResponse::Detected(
                feeds.into_iter().map(Into::into).collect(),
            )),
            DetectorResponse::Processed(feed) => {
                self.repository
                    .cache(FeedCacheData {
                        url: data.url.to_string(),
                        feed: feed.clone(),
                    })
                    .await?;

                Ok(DetectedResponse::Processed(feed))
            }
        }
    }

    pub fn stream(&self) -> BoxStream<Result<String, Error>> {
        self.repository.stream()
    }
}

#[derive(Clone, Debug, Default)]
pub struct FeedListQuery {
    pub folder_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<NonEmptyString>>,
}

#[derive(Clone, Debug)]
pub struct FeedCreate {
    pub url: Url,
    pub title: NonEmptyString,
    pub folder_id: Option<Uuid>,
    pub tags: Option<Vec<NonEmptyString>>,
}

#[derive(Clone, Debug, Default)]
pub struct FeedUpdate {
    pub title: Option<NonEmptyString>,
    pub folder_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<NonEmptyString>>,
}

impl From<FeedUpdate> for FeedUpdateData {
    fn from(value: FeedUpdate) -> Self {
        Self {
            title: value.title.map(String::from),
            folder_id: value.folder_id,
            tags: value.tags,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FeedDetect {
    pub url: Url,
}

#[derive(Clone, Debug, Default)]
pub struct FeedDetected {
    pub url: String,
    pub title: String,
}

impl From<DetectedFeed> for FeedDetected {
    fn from(value: DetectedFeed) -> Self {
        Self {
            url: value.url,
            title: value.title,
        }
    }
}

#[derive(Clone, Debug)]
pub enum DetectedResponse {
    Detected(Vec<FeedDetected>),
    Processed(ProcessedFeed),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ScrapeFeedJob {
    pub url: Url,
}

pub type ScrapeFeedStorage =
    Arc<Mutex<dyn Storage<Job = ScrapeFeedJob, Context = RedisContext, Error = RedisError>>>;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct RefreshFeedsJob(pub DateTime<Utc>);

impl From<DateTime<Utc>> for RefreshFeedsJob {
    fn from(value: DateTime<Utc>) -> Self {
        Self(value)
    }
}
