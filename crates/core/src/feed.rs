use std::sync::Arc;

use colette_scraper::FeedScraper;
pub use colette_scraper::ProcessedFeed;
use futures::stream::BoxStream;
use url::Url;
use uuid::Uuid;

use crate::{
    common::{
        Creatable, Deletable, Findable, IdParams, NonEmptyString, Paginated, TagsLink,
        TagsLinkData, Updatable,
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
    pub pinned: bool,
    pub tags: Option<TagsLink>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct FeedUpdate {
    pub title: Option<Option<NonEmptyString>>,
    pub pinned: Option<bool>,
    pub tags: Option<TagsLink>,
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

pub struct FeedService {
    repository: Arc<dyn FeedRepository>,
    scraper: Arc<dyn FeedScraper>,
}

impl FeedService {
    pub fn new(repository: Arc<dyn FeedRepository>, scraper: Arc<dyn FeedScraper>) -> Self {
        Self {
            repository,
            scraper,
        }
    }

    pub async fn list_feeds(
        &self,
        query: FeedListQuery,
        profile_id: Uuid,
    ) -> Result<Paginated<Feed>, Error> {
        let feeds = self
            .repository
            .list(profile_id, None, None, Some(query.into()))
            .await?;

        Ok(Paginated {
            data: feeds,
            ..Default::default()
        })
    }

    pub async fn get_feed(&self, id: Uuid, profile_id: Uuid) -> Result<Feed, Error> {
        self.repository.find(IdParams::new(id, profile_id)).await
    }

    pub async fn create_feed(&self, mut data: FeedCreate, profile_id: Uuid) -> Result<Feed, Error> {
        let url = data.url.to_string();

        let result = self
            .repository
            .create(FeedCreateData {
                url: url.clone(),
                feed: None,
                pinned: data.pinned,
                tags: data.tags.clone().map(|e| TagsLinkData {
                    data: e.data.into_iter().map(|e| e.into()).collect(),
                    action: e.action,
                }),
                profile_id,
            })
            .await;

        match result {
            Ok(data) => Ok(data),
            Err(Error::Conflict(_)) => {
                let scraped = self.scraper.scrape(&mut data.url)?;

                self.repository
                    .create(FeedCreateData {
                        url,
                        feed: Some(scraped),
                        pinned: data.pinned,
                        tags: data.tags.map(|e| TagsLinkData {
                            data: e.data.into_iter().map(|e| e.into()).collect(),
                            action: e.action,
                        }),
                        profile_id,
                    })
                    .await
            }
            e => e,
        }
    }

    pub async fn update_feed(
        &self,
        id: Uuid,
        data: FeedUpdate,
        profile_id: Uuid,
    ) -> Result<Feed, Error> {
        self.repository
            .update(IdParams::new(id, profile_id), data.into())
            .await
    }

    pub async fn delete_feed(&self, id: Uuid, profile_id: Uuid) -> Result<(), Error> {
        self.repository.delete(IdParams::new(id, profile_id)).await
    }

    pub async fn detect_feeds(&self, _data: FeedDetect) -> Result<Paginated<FeedDetected>, Error> {
        // let urls = self.scraper.detect(&mut data.url)?;
        let urls: Vec<Url> = Vec::new();

        let mut feeds: Vec<FeedDetected> = Vec::new();

        for mut url in urls.into_iter() {
            let feed = self.scraper.scrape(&mut url)?;
            let url = url.to_string();

            feeds.push(FeedDetected {
                url: url.clone(),
                title: feed.title.clone(),
            });

            self.repository.cache(FeedCacheData { url, feed }).await?;
        }

        Ok(Paginated::<FeedDetected> {
            data: feeds,
            cursor: None,
        })
    }
}

#[async_trait::async_trait]
pub trait FeedRepository:
    Findable<Params = IdParams, Output = Result<Feed, Error>>
    + Creatable<Data = FeedCreateData, Output = Result<Feed, Error>>
    + Updatable<Params = IdParams, Data = FeedUpdateData, Output = Result<Feed, Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
{
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
        filters: Option<FeedFindManyFilters>,
    ) -> Result<Vec<Feed>, Error>;

    async fn cache(&self, data: FeedCacheData) -> Result<(), Error>;

    async fn stream(&self) -> Result<BoxStream<Result<String, Error>>, Error>;
}

#[derive(Clone, Debug, Default)]
pub struct FeedFindManyFilters {
    pub pinned: Option<bool>,
    pub tags: Option<Vec<String>>,
}

impl From<FeedListQuery> for FeedFindManyFilters {
    fn from(value: FeedListQuery) -> Self {
        Self {
            pinned: value.pinned,
            tags: value.tags,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FeedCreateData {
    pub url: String,
    pub feed: Option<ProcessedFeed>,
    pub pinned: bool,
    pub tags: Option<TagsLinkData>,
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
    pub tags: Option<TagsLinkData>,
}

impl From<FeedUpdate> for FeedUpdateData {
    fn from(value: FeedUpdate) -> Self {
        Self {
            title: value.title.map(|e| e.map(String::from)),
            pinned: value.pinned,
            tags: value.tags.map(|e| TagsLinkData {
                data: e.data.into_iter().map(|e| e.into()).collect(),
                action: e.action,
            }),
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
