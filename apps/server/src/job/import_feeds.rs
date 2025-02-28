use std::sync::Arc;

use apalis::prelude::Data;
use colette_core::{backup::ImportFeedsJob, feed::ScrapeFeedJob, worker::Storage};
use tokio::sync::Mutex;

pub async fn run(
    job: ImportFeedsJob,
    data: Data<Arc<Mutex<dyn Storage<ScrapeFeedJob>>>>,
) -> Result<(), apalis::prelude::Error> {
    tracing::debug!("Importing {} feeds", job.urls.len());

    let mut storage = data.lock().await;

    for url in job.urls {
        storage
            .push(ScrapeFeedJob { url })
            .await
            .map_err(|e| apalis::prelude::Error::Failed(Arc::new(Box::new(e))))?;
    }

    Ok(())
}
