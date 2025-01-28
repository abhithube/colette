use std::sync::Arc;

use apalis::prelude::{Data, Storage};
use apalis_redis::RedisStorage;
use redis::aio::MultiplexedConnection;
use tokio::sync::Mutex;
use url::Url;

use crate::scrape_feed;

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
    data: Data<Arc<Mutex<RedisStorage<scrape_feed::Job, MultiplexedConnection>>>>,
) -> Result<(), apalis::prelude::Error> {
    tracing::debug!("Importing {} feeds", job.urls.len());

    let mut storage = data.lock().await;

    for url in job.urls {
        storage
            .push(scrape_feed::Job::new(url))
            .await
            .map_err(|e| apalis::prelude::Error::Failed(Arc::new(Box::new(e))))?;
    }

    Ok(())
}
