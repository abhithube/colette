use std::sync::Arc;

use bytes::Bytes;
use colette_backup::BackupManager;
use colette_scraper::feed::FeedScraper;
pub use colette_scraper::feed::ProcessedFeed;
use futures::stream::BoxStream;
use opml::{Body, Outline, OPML};
use url::Url;
use uuid::Uuid;

use crate::{
    common::{Creatable, Deletable, Findable, IdParams, NonEmptyString, Paginated, Updatable},
    tag::TagCreate,
    Tag,
};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Feed {
    pub id: Uuid,
    pub link: String,
    pub title: Option<String>,
    pub original_title: String,
    pub url: Option<String>,
    pub folder_id: Option<Uuid>,
    pub tags: Option<Vec<Tag>>,
    pub unread_count: Option<i64>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FeedCreate {
    pub url: Url,
    pub folder_id: Option<Uuid>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct FeedUpdate {
    pub title: Option<Option<NonEmptyString>>,
    pub folder_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<TagCreate>>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct FeedListQuery {
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

pub struct FeedService {
    repository: Arc<dyn FeedRepository>,
    scraper: Arc<dyn FeedScraper>,
    opml_manager: Arc<dyn BackupManager<T = OPML>>,
}

impl FeedService {
    pub fn new(
        repository: Arc<dyn FeedRepository>,
        scraper: Arc<dyn FeedScraper>,
        opml_manager: Arc<dyn BackupManager<T = OPML>>,
    ) -> Self {
        Self {
            repository,
            scraper,
            opml_manager,
        }
    }

    pub async fn list_feeds(
        &self,
        query: FeedListQuery,
        profile_id: Uuid,
    ) -> Result<Paginated<Feed>, Error> {
        self.repository
            .list(profile_id, None, None, Some(query.into()))
            .await
    }

    pub async fn get_feed(&self, id: Uuid, profile_id: Uuid) -> Result<Feed, Error> {
        self.repository.find(IdParams::new(id, profile_id)).await
    }

    pub async fn create_feed(&self, mut data: FeedCreate, profile_id: Uuid) -> Result<Feed, Error> {
        let url = data.url.to_string();
        let folder_id = Some(data.folder_id);

        let result = self
            .repository
            .create(FeedCreateData {
                url: url.clone(),
                feed: None,
                folder_id,
                profile_id,
            })
            .await;

        match result {
            Ok(data) => Ok(data),
            Err(Error::Conflict(_)) => {
                let scraped = self.scraper.scrape(&mut data.url).await?;

                self.repository
                    .create(FeedCreateData {
                        url,
                        feed: Some(scraped),
                        folder_id,
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

    pub async fn detect_feeds(
        &self,
        mut data: FeedDetect,
    ) -> Result<Paginated<FeedDetected>, Error> {
        let urls = self.scraper.detect(&mut data.url)?;

        let mut feeds: Vec<FeedDetected> = vec![];

        for mut url in urls.into_iter() {
            let feed = self.scraper.scrape(&mut url).await?;
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

    pub async fn import_feeds(&self, raw: Bytes, profile_id: Uuid) -> Result<(), Error> {
        let opml = self.opml_manager.import(raw)?;

        for outline in opml.body.outlines {
            if let Some(xml_url) = outline.xml_url.and_then(|e| Url::parse(&e).ok()) {
                self.create_feed(
                    FeedCreate {
                        url: xml_url,
                        folder_id: None,
                    },
                    profile_id,
                )
                .await?;
            }
        }

        Ok(())
    }

    pub async fn export_feeds(&self, profile_id: Uuid) -> Result<Bytes, Error> {
        let feeds = self
            .list_feeds(FeedListQuery::default(), profile_id)
            .await?;

        let data = feeds
            .data
            .iter()
            .cloned()
            .map(|e| Outline {
                r#type: Some("rss".to_owned()),
                text: e.original_title,
                title: e.title,
                xml_url: e.url,
                html_url: Some(e.link),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        let opml = OPML {
            version: "2.0".to_owned(),
            body: Body { outlines: data },
            ..Default::default()
        };

        self.opml_manager.export(opml).map_err(|e| e.into())
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
        cursor: Option<String>,
        filters: Option<FeedFindManyFilters>,
    ) -> Result<Paginated<Feed>, Error>;

    async fn cache(&self, data: FeedCacheData) -> Result<(), Error>;

    async fn stream(&self) -> Result<BoxStream<Result<(i32, String), Error>>, Error>;

    async fn cleanup(&self) -> Result<(), Error>;
}

#[derive(Clone, Debug, Default)]
pub struct FeedFindManyFilters {
    pub tags: Option<Vec<String>>,
}

impl From<FeedListQuery> for FeedFindManyFilters {
    fn from(value: FeedListQuery) -> Self {
        Self { tags: value.tags }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FeedCreateData {
    pub url: String,
    pub feed: Option<ProcessedFeed>,
    pub folder_id: Option<Option<Uuid>>,
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
    pub folder_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<String>>,
}

impl From<FeedUpdate> for FeedUpdateData {
    fn from(value: FeedUpdate) -> Self {
        Self {
            title: value.title.map(|e| e.map(String::from)),
            folder_id: value.folder_id,
            tags: value
                .tags
                .map(|e| e.into_iter().map(|e| e.title.into()).collect()),
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
