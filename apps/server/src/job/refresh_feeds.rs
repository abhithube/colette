use std::sync::Arc;

use apalis_core::layers::extensions::Data;
use colette_core::feed::{FeedService, RefreshFeedsJob, ScrapeFeedJob, ScrapeFeedStorage};
use futures::StreamExt;

#[derive(Clone)]
pub struct State {
    service: Arc<FeedService>,
    storage: ScrapeFeedStorage,
}

impl State {
    pub fn new(service: Arc<FeedService>, storage: ScrapeFeedStorage) -> Self {
        Self { service, storage }
    }
}

pub async fn run(
    _job: RefreshFeedsJob,
    data: Data<State>,
) -> Result<(), apalis_core::error::Error> {
    tracing::debug!("Refreshing feeds");

    let mut storage = data.storage.lock().await;

    let mut stream = data.service.stream();

    while let Some(Ok(raw)) = stream.next().await {
        storage
            .push(ScrapeFeedJob {
                url: raw.parse().unwrap(),
            })
            .await
            .map_err(|e| apalis_core::error::Error::Failed(Arc::new(Box::new(e))))?;
    }

    Ok(())
}
