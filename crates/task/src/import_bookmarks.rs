use std::sync::Arc;

use apalis::prelude::{Data, Storage};
use apalis_redis::RedisStorage;
use tokio::sync::Mutex;
use url::Url;

use crate::scrape_bookmark;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Job {
    urls: Vec<Url>,
}

impl Job {
    pub fn new(urls: Vec<Url>) -> Self {
        Self { urls }
    }
}

pub async fn run(
    job: Job,
    data: Data<Arc<Mutex<RedisStorage<scrape_bookmark::Job>>>>,
) -> Result<(), apalis::prelude::Error> {
    tracing::debug!("Importing {} bookmarks", job.urls.len());

    let mut storage = data.lock().await;

    for url in job.urls {
        storage
            .push(scrape_bookmark::Job::new(url))
            .await
            .map_err(|e| apalis::prelude::Error::Failed(Arc::new(Box::new(e))))?;
    }

    Ok(())
}
