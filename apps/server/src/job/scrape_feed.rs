use std::sync::Arc;

use apalis::prelude::Data;
use colette_core::{
    feed::ScrapeFeedJob,
    scraper::{FeedCreate, ScraperService},
};

pub async fn run(
    job: ScrapeFeedJob,
    data: Data<Arc<ScraperService>>,
) -> Result<(), apalis::prelude::Error> {
    tracing::debug!("Scraping feed at URL: {}", job.url.as_str());

    data.scrape_feed(FeedCreate { url: job.url })
        .await
        .map_err(|e| apalis::prelude::Error::Failed(Arc::new(Box::new(e))))?;

    Ok(())
}
