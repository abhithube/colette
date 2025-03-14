use std::sync::Arc;

use apalis::prelude::Data;
use colette_core::{
    feed::{FeedService, RefreshFeedsJob, ScrapeFeedJob},
    job::Storage,
};
use futures::StreamExt;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct State {
    service: Arc<FeedService>,
    storage: Arc<Mutex<dyn Storage<ScrapeFeedJob>>>,
}

impl State {
    pub fn new(service: Arc<FeedService>, storage: Arc<Mutex<dyn Storage<ScrapeFeedJob>>>) -> Self {
        Self { service, storage }
    }
}

pub async fn run(_job: RefreshFeedsJob, data: Data<State>) -> Result<(), apalis::prelude::Error> {
    tracing::debug!("Refreshing feeds");

    let mut storage = data.storage.lock().await;

    let mut stream = data
        .service
        .stream()
        .await
        .map_err(|e| apalis::prelude::Error::Failed(Arc::new(Box::new(e))))?;

    while let Some(Ok(url)) = stream.next().await {
        storage
            .push(ScrapeFeedJob { url })
            .await
            .map_err(|e| apalis::prelude::Error::Failed(Arc::new(Box::new(e))))?;
    }

    Ok(())
}
