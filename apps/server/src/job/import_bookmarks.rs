use std::sync::Arc;

use apalis::prelude::Data;
use colette_core::{backup::ImportBookmarksJob, bookmark::ScrapeBookmarkJob, job::Storage};
use tokio::sync::Mutex;

pub async fn run(
    job: ImportBookmarksJob,
    data: Data<Arc<Mutex<dyn Storage<ScrapeBookmarkJob>>>>,
) -> Result<(), apalis::prelude::Error> {
    tracing::debug!("Importing {} bookmarks", job.urls.len());

    let mut storage = data.lock().await;

    for url in job.urls {
        storage
            .push(ScrapeBookmarkJob {
                url,
                user_id: job.user_id.clone(),
            })
            .await
            .map_err(|e| apalis::prelude::Error::Failed(Arc::new(Box::new(e))))?;
    }

    Ok(())
}
