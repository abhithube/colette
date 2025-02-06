use std::sync::Arc;

use apalis::prelude::Data;
use colette_core::feed::{FeedPersist, FeedService, ScrapeFeedJob};

pub async fn run(
    job: ScrapeFeedJob,
    data: Data<Arc<FeedService>>,
) -> Result<(), apalis::prelude::Error> {
    tracing::debug!("Scraping feed at URL: {}", job.url.as_str());

    data.scrape_and_persist_feed(FeedPersist { url: job.url })
        .await
        .map_err(|e| apalis::prelude::Error::Failed(Arc::new(Box::new(e))))?;

    Ok(())
}
