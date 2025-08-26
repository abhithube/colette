use std::sync::Arc;

use colette_common::RepositoryError;
use colette_http::HttpClient;
use colette_ingestion::{Feed, FeedBatch, FeedEntry, FeedError, FeedRepository};
use colette_scraper::feed::FeedScraper;
use url::Url;
use uuid::ContextV7;

use crate::Handler;

#[derive(Debug, Clone)]
pub struct ScrapeFeedCommand {
    pub url: Url,
}

pub struct ScrapeFeedHandler<FR: FeedRepository, HC: HttpClient> {
    feed_repository: FR,
    feed_scraper: Arc<FeedScraper<HC>>,
}

impl<FR: FeedRepository, HC: HttpClient> ScrapeFeedHandler<FR, HC> {
    pub fn new(feed_repository: FR, feed_scraper: Arc<FeedScraper<HC>>) -> Self {
        Self {
            feed_repository,
            feed_scraper,
        }
    }
}

impl<FR: FeedRepository, HC: HttpClient> Handler<ScrapeFeedCommand> for ScrapeFeedHandler<FR, HC> {
    type Response = FeedCreated;
    type Error = ScrapeFeedError;

    async fn handle(&self, mut cmd: ScrapeFeedCommand) -> Result<Self::Response, Self::Error> {
        let feed = match self.feed_repository.find_by_source_url(&cmd.url).await? {
            Some(feed) => feed,
            None => {
                let processed = match self.feed_scraper.scrape(&mut cmd.url).await {
                    Ok(processed) => Ok(processed),
                    Err(e) => {
                        self.feed_repository.mark_as_failed(cmd.url.clone()).await?;

                        Err(e)
                    }
                }?;

                let is_custom = processed.link == cmd.url;

                let feed = Feed::new(
                    cmd.url,
                    processed.link,
                    processed.title,
                    processed.description,
                    is_custom,
                );

                let uuid_ctx = ContextV7::new();
                let feed_entries = processed
                    .entries
                    .into_iter()
                    .map(|e| {
                        FeedEntry::new(
                            &uuid_ctx,
                            e.link,
                            e.title,
                            e.published,
                            e.description,
                            e.author,
                            e.thumbnail,
                        )
                    })
                    .collect();

                let created = feed.clone();

                self.feed_repository
                    .upsert(FeedBatch { feed, feed_entries })
                    .await?;

                created
            }
        };

        Ok(FeedCreated {
            source_url: feed.source_url().to_owned(),
            link: feed.link().to_owned(),
            title: feed.title().to_owned(),
            description: feed.description().map(ToOwned::to_owned),
        })
    }
}

#[derive(Debug, Clone)]
pub struct FeedCreated {
    pub source_url: Url,
    pub link: Url,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ScrapeFeedError {
    #[error(transparent)]
    Feed(#[from] FeedError),

    #[error(transparent)]
    Scraper(#[from] colette_scraper::feed::FeedError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
