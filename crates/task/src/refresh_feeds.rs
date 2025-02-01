use std::sync::Arc;

use apalis::prelude::Data;
use chrono::{DateTime, Utc};
use colette_core::feed::FeedService;
use futures::StreamExt;
use tokio::sync::Mutex;
use url::Url;

use crate::{scrape_feed, Storage};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Job(pub DateTime<Utc>);

impl From<DateTime<Utc>> for Job {
    fn from(value: DateTime<Utc>) -> Self {
        Self(value)
    }
}

#[derive(Clone)]
pub struct State {
    service: Arc<FeedService>,
    storage: Arc<Mutex<dyn Storage<Job = scrape_feed::Job>>>,
}

impl State {
    pub fn new(
        service: Arc<FeedService>,
        storage: Arc<Mutex<dyn Storage<Job = scrape_feed::Job>>>,
    ) -> Self {
        Self { service, storage }
    }
}

pub async fn run(_job: Job, data: Data<State>) -> Result<(), apalis::prelude::Error> {
    tracing::debug!("Refreshing feeds");

    let mut storage = data.storage.lock().await;

    let mut stream = data.service.stream();

    while let Some(Ok(raw)) = stream.next().await {
        let url = Url::parse(&raw).unwrap();

        storage
            .push(scrape_feed::Job { url })
            .await
            .map_err(|e| apalis::prelude::Error::Failed(Arc::new(Box::new(e))))?;
    }

    Ok(())
}
