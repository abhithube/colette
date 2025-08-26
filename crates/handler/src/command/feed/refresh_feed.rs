use std::sync::Arc;

use colette_common::RepositoryError;
use colette_http::HttpClient;
use colette_ingestion::{FeedBatch, FeedEntry, FeedError, FeedId, FeedRepository};
use colette_scraper::feed::FeedScraper;
use uuid::ContextV7;

use crate::Handler;

#[derive(Debug, Clone)]
pub struct RefreshFeedCommand {
    pub id: FeedId,
}

pub struct RefreshFeedHandler<FR: FeedRepository, HC: HttpClient> {
    feed_repository: FR,

    feed_scraper: Arc<FeedScraper<HC>>,
}

impl<FR: FeedRepository, HC: HttpClient> RefreshFeedHandler<FR, HC> {
    pub fn new(feed_repository: FR, feed_scraper: Arc<FeedScraper<HC>>) -> Self {
        Self {
            feed_repository,

            feed_scraper,
        }
    }
}

impl<FR: FeedRepository, HC: HttpClient> Handler<RefreshFeedCommand>
    for RefreshFeedHandler<FR, HC>
{
    type Response = ();
    type Error = RefreshFeedError;

    async fn handle(&self, cmd: RefreshFeedCommand) -> Result<Self::Response, Self::Error> {
        let mut feed =
            self.feed_repository
                .find_by_id(cmd.id)
                .await?
                .ok_or(RefreshFeedError::Feed(FeedError::NotFound(
                    cmd.id.as_inner(),
                )))?;

        let mut source_url = feed.source_url().to_owned();

        let processed = match self.feed_scraper.scrape(&mut source_url).await {
            Ok(processed) => Ok(processed),
            Err(e) => {
                self.feed_repository
                    .mark_as_failed(source_url.clone())
                    .await?;

                Err(e)
            }
        }?;

        feed.set_link(processed.link);
        feed.set_title(processed.title);
        if let Some(description) = processed.description {
            feed.set_description(description);
        } else {
            feed.remove_description();
        }

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

        self.feed_repository
            .upsert(FeedBatch { feed, feed_entries })
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RefreshFeedError {
    #[error(transparent)]
    Feed(#[from] FeedError),

    #[error(transparent)]
    Scraper(#[from] colette_scraper::feed::FeedError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
