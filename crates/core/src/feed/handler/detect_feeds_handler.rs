use std::sync::Arc;

use bytes::Buf;
use colette_http::HttpClient;
use colette_scraper::feed::FeedScraper;
use url::Url;

use crate::{Handler, RepositoryError, feed::FeedDetected};

#[derive(Debug, Clone)]
pub struct DetectFeedsCommand {
    pub url: Url,
}

pub struct DetectFeedsHandler {
    http_client: Box<dyn HttpClient>,
    feed_scraper: Arc<FeedScraper>,
}

impl DetectFeedsHandler {
    pub fn new(http_client: impl HttpClient, feed_scraper: Arc<FeedScraper>) -> Self {
        Self {
            http_client: Box::new(http_client),
            feed_scraper,
        }
    }
}

#[async_trait::async_trait]
impl Handler<DetectFeedsCommand> for DetectFeedsHandler {
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
