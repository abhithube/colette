use std::sync::Arc;

use apalis_core::layers::extensions::Data;
use colette_core::{
    feed::ScrapeFeedJob,
    scraper::{FeedCreate, ScraperService},
};

pub async fn run(
    job: ScrapeFeedJob,
    data: Data<Arc<ScraperService>>,
) -> Result<(), apalis_core::error::Error> {
    tracing::debug!("Scraping feed at URL: {}", job.url.as_str());

    data.scrape_feed(FeedCreate { url: job.url })
        .await
        .map_err(|e| apalis_core::error::Error::Failed(Arc::new(Box::new(e))))?;

    Ok(())
}
