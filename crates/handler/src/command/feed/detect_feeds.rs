use std::sync::Arc;

use bytes::Buf;
use colette_common::RepositoryError;
use colette_http::HttpClient;
use colette_ingestion::FeedDetected;
use colette_scraper::feed::FeedScraper;
use url::Url;

use crate::Handler;

#[derive(Debug, Clone)]
pub struct DetectFeedsCommand {
    pub url: Url,
}

pub struct DetectFeedsHandler<HC: HttpClient> {
    http_client: HC,
    feed_scraper: Arc<FeedScraper<HC>>,
}

impl<HC: HttpClient> DetectFeedsHandler<HC> {
    pub fn new(http_client: HC, feed_scraper: Arc<FeedScraper<HC>>) -> Self {
        Self {
            http_client,
            feed_scraper,
        }
    }
}

impl<HC: HttpClient> Handler<DetectFeedsCommand> for DetectFeedsHandler<HC> {
    type Response = Vec<FeedDetected>;
    type Error = DetectFeedsError;

    async fn handle(&self, mut cmd: DetectFeedsCommand) -> Result<Self::Response, Self::Error> {
        match self.feed_scraper.scrape(&mut cmd.url).await {
            Ok(processed) => {
                let detected = vec![FeedDetected {
                    url: cmd.url,
                    title: processed.title,
                }];

                Ok(detected)
            }
            Err(colette_scraper::feed::FeedError::Unsupported) => {
                let body = self.http_client.get(&cmd.url).await?;

                let metadata = colette_meta::parse_metadata(body.reader())
                    .map_err(|_| colette_scraper::feed::FeedError::Unsupported)?;

                let detected = metadata
                    .feeds
                    .into_iter()
                    .map(|feed| FeedDetected {
                        url: feed.href.parse().unwrap(),
                        title: feed.title,
                    })
                    .collect::<Vec<_>>();

                Ok(detected)
            }
            Err(e) => Err(DetectFeedsError::Scraper(e)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DetectFeedsError {
    #[error(transparent)]
    Scraper(#[from] colette_scraper::feed::FeedError),

    #[error(transparent)]
    Http(#[from] colette_http::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Utf(#[from] std::str::Utf8Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
