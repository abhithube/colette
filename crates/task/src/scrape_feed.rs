use std::sync::Arc;

use apalis::prelude::Data;
use colette_core::scraper::{FeedCreate, ScraperService};
use url::Url;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Job {
    url: Url,
}

impl Job {
    pub fn new(url: Url) -> Self {
        Self { url }
    }
}

pub async fn run(job: Job, data: Data<Arc<ScraperService>>) -> Result<(), apalis::prelude::Error> {
    tracing::debug!("Scraping feed at URL: {}", job.url.as_str());

    data.scrape_feed(FeedCreate { url: job.url })
        .await
        .map_err(|e| apalis::prelude::Error::Failed(Arc::new(Box::new(e))))?;

    Ok(())
}
