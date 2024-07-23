use std::sync::Arc;

use chrono::{DateTime, Utc};
use url::Url;
use uuid::Uuid;

use crate::{
    common::{FindOneParams, Paginated, SendableStream, Session},
    utils::scraper::{self, ExtractorQuery, Scraper},
};

#[derive(Clone, Debug)]
pub struct Feed {
    pub id: Uuid,
    pub link: String,
    pub title: String,
    pub url: Option<String>,
    pub custom_title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub unread_count: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct CreateFeed {
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct UpdateFeed {
    pub title: Option<String>,
}

#[derive(Clone, Debug)]
pub struct FeedExtractorOptions<'a> {
    pub feed_link_queries: Vec<ExtractorQuery<'a>>,
    pub feed_title_queries: Vec<ExtractorQuery<'a>>,
    pub feed_entries_selector: &'a str,
    pub entry_link_queries: Vec<ExtractorQuery<'a>>,
    pub entry_title_queries: Vec<ExtractorQuery<'a>>,
    pub entry_published_queries: Vec<ExtractorQuery<'a>>,
    pub entry_description_queries: Vec<ExtractorQuery<'a>>,
    pub entry_author_queries: Vec<ExtractorQuery<'a>>,
    pub entry_thumbnail_queries: Vec<ExtractorQuery<'a>>,
}

#[derive(Clone, Debug)]
pub struct ExtractedFeed {
    pub link: Option<String>,
    pub title: Option<String>,
    pub entries: Vec<ExtractedEntry>,
}

#[derive(Clone, Debug)]
pub struct ExtractedEntry {
    pub link: Option<String>,
    pub title: Option<String>,
    pub published: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ProcessedFeed {
    pub link: Url,
    pub title: String,
    pub entries: Vec<ProcessedEntry>,
}

#[derive(Clone, Debug)]
pub struct ProcessedEntry {
    pub link: Url,
    pub title: String,
    pub published: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail: Option<Url>,
}

#[derive(Clone, Debug)]
pub struct BackupFeed {
    pub title: String,
    pub xml_url: Url,
    pub html_url: Option<Url>,
}

#[async_trait::async_trait]
pub trait FeedsRepository: Send + Sync {
    async fn find_many(&self, params: FeedsFindManyParams) -> Result<Vec<Feed>, Error>;

    async fn find_one(&self, params: FindOneParams) -> Result<Feed, Error>;

    async fn create(&self, data: FeedsCreateData) -> Result<Feed, Error>;

    async fn update(&self, params: FindOneParams, data: FeedsUpdateData) -> Result<Feed, Error>;

    async fn delete(&self, params: FindOneParams) -> Result<(), Error>;

    fn iterate(&self) -> SendableStream<Result<(i64, String), Error>>;

    async fn cleanup(&self) -> Result<(), Error>;
}

pub struct FeedsService {
    repo: Arc<dyn FeedsRepository>,
    scraper: Arc<dyn Scraper<ProcessedFeed>>,
}

impl FeedsService {
    pub fn new(repo: Arc<dyn FeedsRepository>, scraper: Arc<dyn Scraper<ProcessedFeed>>) -> Self {
        Self { repo, scraper }
    }

    pub async fn list(&self, session: Session) -> Result<Paginated<Feed>, Error> {
        let feeds = self
            .repo
            .find_many(FeedsFindManyParams {
                profile_id: session.profile_id,
            })
            .await?;

        let paginated = Paginated::<Feed> {
            has_more: false,
            data: feeds,
        };

        Ok(paginated)
    }

    pub async fn get(&self, id: Uuid, session: Session) -> Result<Feed, Error> {
        let feed = self
            .repo
            .find_one(FindOneParams {
                id,
                profile_id: session.profile_id,
            })
            .await?;

        Ok(feed)
    }

    pub async fn create(&self, mut data: CreateFeed, session: Session) -> Result<Feed, Error> {
        let scraped = self.scraper.scrape(&mut data.url)?;

        let feed = self
            .repo
            .create(FeedsCreateData {
                url: data.url,
                feed: scraped,
                profile_id: session.profile_id,
            })
            .await?;

        Ok(feed)
    }

    pub async fn update(
        &self,
        id: Uuid,
        data: UpdateFeed,
        session: Session,
    ) -> Result<Feed, Error> {
        let feed = self
            .repo
            .update(
                FindOneParams {
                    id,
                    profile_id: session.profile_id,
                },
                data.into(),
            )
            .await?;

        Ok(feed)
    }

    pub async fn delete(&self, id: Uuid, session: Session) -> Result<(), Error> {
        self.repo
            .delete(FindOneParams {
                id,
                profile_id: session.profile_id,
            })
            .await?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct FeedsFindManyParams {
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct FeedsCreateData {
    pub url: String,
    pub feed: ProcessedFeed,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct FeedsUpdateData {
    pub custom_title: Option<String>,
}

impl From<UpdateFeed> for FeedsUpdateData {
    fn from(value: UpdateFeed) -> Self {
        Self {
            custom_title: value.title,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("feed not found with id: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Scraper(#[from] scraper::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
