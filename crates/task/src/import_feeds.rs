use std::sync::Arc;

use apalis::prelude::Data;
use tokio::sync::Mutex;
use url::Url;

use crate::{scrape_feed, Storage};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Job {
    pub urls: Vec<Url>,
}

pub async fn run(
    job: Job,
    data: Data<Arc<Mutex<dyn Storage<Job = scrape_feed::Job>>>>,
) -> Result<(), apalis::prelude::Error> {
    tracing::debug!("Importing {} feeds", job.urls.len());

    let mut storage = data.lock().await;

    for url in job.urls {
        storage
            .push(scrape_feed::Job { url })
            .await
            .map_err(|e| apalis::prelude::Error::Failed(Arc::new(Box::new(e))))?;
    }

    Ok(())
}
