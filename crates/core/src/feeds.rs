use std::sync::Arc;

use chrono::{DateTime, Utc};
use url::Url;

use crate::{
    common::{FindOneParams, Paginated, Session},
    scraper::{self, Scraper},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("feed not found with id: {0}")]
    NotFound(String),

    #[error(transparent)]
    Scraper(#[from] scraper::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug)]
pub struct Feed {
    pub id: String,
    pub link: String,
    pub title: String,
    pub url: Option<String>,
    pub custom_title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub unread_count: Option<i64>,
}

#[derive(Debug)]
pub struct CreateFeed {
    pub url: String,
}

#[derive(Debug)]
pub struct ExtractorOptions {
    pub feed_link_expr: &'static [&'static str],
    pub feed_title_expr: &'static [&'static str],
    pub feed_entries_expr: &'static [&'static str],
    pub entry_link_expr: &'static [&'static str],
    pub entry_title_expr: &'static [&'static str],
    pub entry_published_expr: &'static [&'static str],
    pub entry_description_expr: &'static [&'static str],
    pub entry_author_expr: &'static [&'static str],
    pub entry_thumbnail_expr: &'static [&'static str],
}

#[derive(Debug)]
pub struct ExtractedFeed {
    pub link: Option<String>,
    pub title: Option<String>,
    pub entries: Vec<ExtractedEntry>,
}

#[derive(Debug)]
pub struct ExtractedEntry {
    pub link: Option<String>,
    pub title: Option<String>,
    pub published: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProcessedFeed {
    pub link: Url,
    pub title: String,
    pub entries: Vec<ProcessedEntry>,
}

#[derive(Debug, Clone)]
pub struct ProcessedEntry {
    pub link: Url,
    pub title: String,
    pub published: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail: Option<Url>,
}

#[async_trait::async_trait]
pub trait FeedsRepository {
    async fn find_many(&self, params: FeedFindManyParams) -> Result<Vec<Feed>, Error>;

    async fn find_one(&self, params: FindOneParams) -> Result<Feed, Error>;

    async fn create(&self, data: FeedCreateData) -> Result<Feed, Error>;

    async fn delete(&self, params: FindOneParams) -> Result<(), Error>;
}

pub struct FeedFindManyParams {
    pub profile_id: String,
}

pub struct FeedCreateData {
    pub url: String,
    pub feed: ProcessedFeed,
    pub profile_id: String,
}

pub struct FeedsService {
    repo: Arc<dyn FeedsRepository + Send + Sync>,
    scraper: Arc<dyn Scraper<ProcessedFeed> + Send + Sync>,
}

impl FeedsService {
    pub fn new(
        feeds_repo: Arc<dyn FeedsRepository + Send + Sync>,
        scraper: Arc<dyn Scraper<ProcessedFeed> + Send + Sync>,
    ) -> Self {
        Self {
            repo: feeds_repo,
            scraper,
        }
    }

    pub async fn list(&self, session: Session) -> Result<Paginated<Feed>, Error> {
        let params = FeedFindManyParams {
            profile_id: session.profile_id,
        };
        let feeds = self.repo.find_many(params).await?;

        let paginated = Paginated::<Feed> {
            has_more: false,
            data: feeds,
        };

        Ok(paginated)
    }

    pub async fn get(&self, id: String, session: Session) -> Result<Feed, Error> {
        let params = FindOneParams {
            id,
            profile_id: session.profile_id,
        };
        let feed = self.repo.find_one(params).await?;

        Ok(feed)
    }

    pub async fn create(&self, data: CreateFeed, session: Session) -> Result<Feed, Error> {
        let scraped = self.scraper.scrape(&data.url).await?;

        let data = FeedCreateData {
            url: data.url,
            feed: scraped,
            profile_id: session.profile_id,
        };
        let feed = self.repo.create(data).await?;

        Ok(feed)
    }

    pub async fn delete(&self, id: String, session: Session) -> Result<(), Error> {
        let params = FindOneParams {
            id,
            profile_id: session.profile_id,
        };
        self.repo.delete(params).await?;

        Ok(())
    }
}
