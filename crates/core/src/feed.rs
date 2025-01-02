pub use colette_scraper::feed::ProcessedFeed;
use colette_scraper::feed::{DetectorResponse, FeedDetector};
use futures::stream::BoxStream;
use url::Url;
use uuid::Uuid;

use crate::{
    common::{
        Creatable, Deletable, Findable, IdParams, NonEmptyString, NonEmptyVec, Paginated, Updatable,
    },
    Tag,
};

#[derive(Clone, Debug, Default)]
pub struct Feed {
    pub id: Uuid,
    pub link: String,
    pub title: Option<String>,
    pub pinned: bool,
    pub original_title: String,
    pub url: Option<String>,
    pub tags: Option<Vec<Tag>>,
    pub unread_count: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct FeedCreate {
    pub url: Url,
    pub title: Option<NonEmptyString>,
    pub pinned: Option<bool>,
    pub tags: Option<NonEmptyVec<NonEmptyString>>,
}

#[derive(Clone, Debug, Default)]
pub struct FeedUpdate {
    pub title: Option<Option<NonEmptyString>>,
    pub pinned: Option<bool>,
    pub tags: Option<NonEmptyVec<NonEmptyString>>,
}

#[derive(Clone, Debug, Default)]
pub struct FeedListQuery {
    pub pinned: Option<bool>,
    pub tags: Option<Vec<String>>,
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

impl From<colette_scraper::feed::DetectedFeed> for FeedDetected {
    fn from(value: colette_scraper::feed::DetectedFeed) -> Self {
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

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    pub id: Uuid,
    pub title: String,
}

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
                pinned: query.pinned,
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
                title: data.title.map(String::from),
                pinned: data.pinned,
                tags: data
                    .tags
                    .map(|e| Vec::from(e).into_iter().map(String::from).collect()),
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

    pub async fn stream(&self) -> Result<BoxStream<String>, Error> {
        self.repository.stream().await
    }
}

#[async_trait::async_trait]
pub trait FeedRepository:
    Findable<Params = FeedFindParams, Output = Result<Vec<Feed>, Error>>
    + Creatable<Data = FeedCreateData, Output = Result<Uuid, Error>>
    + Updatable<Params = IdParams, Data = FeedUpdateData, Output = Result<(), Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
    + 'static
{
    async fn cache(&self, data: FeedCacheData) -> Result<(), Error>;

    async fn stream(&self) -> Result<BoxStream<String>, Error>;
}

#[derive(Clone, Debug, Default)]
pub struct FeedFindParams {
    pub id: Option<Uuid>,
    pub pinned: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub user_id: Uuid,
    pub limit: Option<u64>,
    pub cursor: Option<Cursor>,
}

#[derive(Clone, Debug, Default)]
pub struct FeedCreateData {
    pub url: String,
    pub title: Option<String>,
    pub pinned: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub user_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct FeedCacheData {
    pub url: String,
    pub feed: ProcessedFeed,
}

#[derive(Clone, Debug, Default)]
pub struct FeedUpdateData {
    pub title: Option<Option<String>>,
    pub pinned: Option<bool>,
    pub tags: Option<Vec<String>>,
}

impl From<FeedUpdate> for FeedUpdateData {
    fn from(value: FeedUpdate) -> Self {
        Self {
            title: value.title.map(|e| e.map(String::from)),
            pinned: value.pinned,
            tags: value
                .tags
                .map(|e| Vec::from(e).into_iter().map(String::from).collect()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("feed not found with id: {0}")]
    NotFound(Uuid),

    #[error("feed not cached with URL: {0}")]
    Conflict(String),

    #[error(transparent)]
    Scraper(#[from] colette_scraper::Error),

    #[error(transparent)]
    Backup(#[from] colette_backup::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
