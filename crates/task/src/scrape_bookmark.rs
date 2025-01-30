use std::sync::Arc;

use apalis::prelude::Data;
use colette_core::scraper::{BookmarkCreate, ScraperService};
use url::Url;
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Job {
    url: Url,
    user_id: Uuid,
}

impl Job {
    pub fn new(url: Url, user_id: Uuid) -> Self {
        Self { url, user_id }
    }
}

pub async fn run(job: Job, data: Data<Arc<ScraperService>>) -> Result<(), apalis::prelude::Error> {
    tracing::debug!("Scraping bookmark at URL: {}", job.url.as_str());

    data.scrape_bookmark(BookmarkCreate {
        url: job.url,
        user_id: job.user_id,
    })
    .await
    .map_err(|e| apalis::prelude::Error::Failed(Arc::new(Box::new(e))))?;

    Ok(())
}
