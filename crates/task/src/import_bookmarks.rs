use std::sync::Arc;

use apalis::prelude::Data;
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;

use crate::{scrape_bookmark, Storage};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Job {
    urls: Vec<Url>,
    user_id: Uuid,
}

impl Job {
    pub fn new(urls: Vec<Url>, user_id: Uuid) -> Self {
        Self { urls, user_id }
    }
}

pub async fn run(
    job: Job,
    data: Data<Arc<Mutex<dyn Storage<Job = scrape_bookmark::Job>>>>,
) -> Result<(), apalis::prelude::Error> {
    tracing::debug!("Importing {} bookmarks", job.urls.len());

    let mut storage = data.lock().await;

    for url in job.urls {
        storage
            .push(scrape_bookmark::Job::new(url, job.user_id))
            .await
            .map_err(|e| apalis::prelude::Error::Failed(Arc::new(Box::new(e))))?;
    }

    Ok(())
}
