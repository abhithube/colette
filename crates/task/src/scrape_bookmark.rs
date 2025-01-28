use std::sync::Arc;

use apalis::prelude::Data;
use colette_core::scraper::{BookmarkCreate, ScraperService};
use url::Url;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Job {
    url: Url,
}

impl Job {
    pub fn new(url: Url) -> Self {
        Self { url }
    }
}

pub async fn run(job: Job, data: Data<Arc<ScraperService>>) -> Result<(), apalis::prelude::Error> {
    data.scrape_bookmark(BookmarkCreate { url: job.url })
        .await
        .map_err(|e| apalis::prelude::Error::Failed(Arc::new(Box::new(e))))?;

    Ok(())
}
