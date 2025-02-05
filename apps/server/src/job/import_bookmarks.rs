use std::sync::Arc;

use apalis_core::layers::extensions::Data;
use colette_core::{
    backup::ImportBookmarksJob,
    bookmark::{ScrapeBookmarkJob, ScrapeBookmarkStorage},
};

pub async fn run(
    job: ImportBookmarksJob,
    data: Data<ScrapeBookmarkStorage>,
) -> Result<(), apalis_core::error::Error> {
    tracing::debug!("Importing {} bookmarks", job.urls.len());

    let mut storage = data.lock().await;

    for url in job.urls {
        storage
            .push(ScrapeBookmarkJob {
                url,
                user_id: job.user_id,
            })
            .await
            .map_err(|e| apalis_core::error::Error::Failed(Arc::new(Box::new(e))))?;
    }

    Ok(())
}
