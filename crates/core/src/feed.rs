use colette_scraper::feed::FeedDetector;
pub use colette_scraper::feed::ProcessedFeed;
use dyn_clone::DynClone;
use futures::stream::BoxStream;
use url::Url;
use uuid::Uuid;

use crate::{
    common::{
        Creatable, Deletable, Findable, IdParams, NonEmptyString, NonEmptyVec, Paginated, Updatable,
    },
    Tag,
};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FeedCreate {
    pub url: Url,
    pub pinned: Option<bool>,
    pub tags: Option<NonEmptyVec<NonEmptyString>>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct FeedUpdate {
    pub title: Option<Option<NonEmptyString>>,
    pub pinned: Option<bool>,
    pub tags: Option<NonEmptyVec<NonEmptyString>>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct FeedListQuery {
    pub pinned: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FeedDetect {
    pub url: Url,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct FeedDetected {
    pub url: String,
    pub title: String,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    pub id: Uuid,
    pub title: String,
}

#[derive(Clone)]
pub struct FeedService {
    repository: Box<dyn FeedRepository>,
    detector: Box<dyn FeedDetector>,
}

impl FeedService {
    pub fn new(repository: Box<dyn FeedRepository>, detector: Box<dyn FeedDetector>) -> Self {
        Self {
            repository,
            detector,
        }
    }

    pub async fn list_feeds(
        &self,
        query: FeedListQuery,
        profile_id: Uuid,
    ) -> Result<Paginated<Feed>, Error> {
        let feeds = self
            .repository
            .find(FeedFindParams {
                pinned: query.pinned,
                tags: query.tags,
                profile_id,
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: feeds,
            ..Default::default()
        })
    }

    pub async fn get_feed(&self, id: Uuid, profile_id: Uuid) -> Result<Feed, Error> {
        let mut feeds = self
            .repository
            .find(FeedFindParams {
                id: Some(id),
                profile_id,
                ..Default::default()
            })
            .await?;
        if feeds.is_empty() {
            return Err(Error::NotFound(id));
        }

        Ok(feeds.swap_remove(0))
    }

    pub async fn create_feed(&self, data: FeedCreate, profile_id: Uuid) -> Result<Feed, Error> {
        let id = self
            .repository
            .create(FeedCreateData {
                url: data.url.to_string(),
                pinned: data.pinned,
                tags: data
                    .tags
                    .clone()
                    .map(|e| Vec::from(e).into_iter().map(String::from).collect()),
                profile_id,
            })
            .await?;

        self.get_feed(id, profile_id).await
    }

    pub async fn update_feed(
        &self,
        id: Uuid,
        data: FeedUpdate,
        profile_id: Uuid,
    ) -> Result<Feed, Error> {
        self.repository
            .update(IdParams::new(id, profile_id), data.into())
            .await?;

        self.get_feed(id, profile_id).await
    }

    pub async fn delete_feed(&self, id: Uuid, profile_id: Uuid) -> Result<(), Error> {
        self.repository.delete(IdParams::new(id, profile_id)).await
    }

    pub async fn detect_feeds(&self, data: FeedDetect) -> Result<Paginated<FeedDetected>, Error> {
        let detected = self.detector.detect(data.url).await?;

        let mut feeds: Vec<FeedDetected> = Vec::new();
        let mut data: Vec<FeedCacheData> = Vec::new();

        for (url, feed) in detected {
            let url = url.to_string();

            feeds.push(FeedDetected {
                url: url.clone(),
                title: feed.title.clone(),
            });
            data.push(FeedCacheData { url, feed });
        }

        self.repository.cache(data).await?;

        Ok(Paginated::<FeedDetected> {
            data: feeds,
            cursor: None,
        })
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
    + DynClone
{
    async fn cache(&self, data: Vec<FeedCacheData>) -> Result<(), Error>;

    async fn stream(&self) -> Result<BoxStream<String>, Error>;
}

dyn_clone::clone_trait_object!(FeedRepository);

#[derive(Clone, Debug, Default)]
pub struct FeedFindParams {
    pub id: Option<Uuid>,
    pub pinned: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub profile_id: Uuid,
    pub limit: Option<u64>,
    pub cursor: Option<Cursor>,
}

#[derive(Clone, Debug, Default)]
pub struct FeedCreateData {
    pub url: String,
    pub pinned: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub profile_id: Uuid,
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
