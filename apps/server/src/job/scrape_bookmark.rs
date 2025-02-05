use std::sync::Arc;

use apalis_core::layers::extensions::Data;
use colette_core::{
    bookmark::ScrapeBookmarkJob,
    scraper::{BookmarkCreate, ScraperService},
};

pub async fn run(
    job: ScrapeBookmarkJob,
    data: Data<Arc<ScraperService>>,
) -> Result<(), apalis_core::error::Error> {
    tracing::debug!("Scraping bookmark at URL: {}", job.url.as_str());

    data.scrape_bookmark(BookmarkCreate {
        url: job.url,
        user_id: job.user_id,
    })
    .await
    .map_err(|e| apalis_core::error::Error::Failed(Arc::new(Box::new(e))))?;

    Ok(())
}
