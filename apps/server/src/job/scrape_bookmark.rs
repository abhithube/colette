use std::sync::Arc;

use apalis::prelude::Data;
use colette_core::{
    bookmark::ScrapeBookmarkJob,
    scraper::{BookmarkCreate, ScraperService},
};

pub async fn run(
    job: ScrapeBookmarkJob,
    data: Data<Arc<ScraperService>>,
) -> Result<(), apalis::prelude::Error> {
    tracing::debug!("Scraping bookmark at URL: {}", job.url.as_str());

    data.scrape_bookmark(BookmarkCreate {
        url: job.url,
        user_id: job.user_id,
    })
    .await
    .map_err(|e| apalis::prelude::Error::Failed(Arc::new(Box::new(e))))?;

    Ok(())
}
