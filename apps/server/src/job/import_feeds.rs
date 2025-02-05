use std::sync::Arc;

use apalis_core::layers::extensions::Data;
use colette_core::{
    backup::ImportFeedsJob,
    feed::{ScrapeFeedJob, ScrapeFeedStorage},
};

pub async fn run(
    job: ImportFeedsJob,
    data: Data<ScrapeFeedStorage>,
) -> Result<(), apalis_core::error::Error> {
    tracing::debug!("Importing {} feeds", job.urls.len());

    let mut storage = data.lock().await;

    for url in job.urls {
        storage
            .push(ScrapeFeedJob { url })
            .await
            .map_err(|e| apalis_core::error::Error::Failed(Arc::new(Box::new(e))))?;
    }

    Ok(())
}
