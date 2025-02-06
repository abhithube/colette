use std::sync::Arc;

use apalis::prelude::Data;
use colette_core::bookmark::{BookmarkPersist, BookmarkService, ScrapeBookmarkJob};

pub async fn run(
    job: ScrapeBookmarkJob,
    data: Data<Arc<BookmarkService>>,
) -> Result<(), apalis::prelude::Error> {
    tracing::debug!("Scraping bookmark at URL: {}", job.url.as_str());

    data.scrape_and_persist_bookmark(BookmarkPersist {
        url: job.url,
        user_id: job.user_id,
    })
    .await
    .map_err(|e| apalis::prelude::Error::Failed(Arc::new(Box::new(e))))?;

    Ok(())
}
