use std::sync::Arc;

use apalis::prelude::Data;
use colette_core::{backup::ImportBookmarksJob, bookmark::ScrapeBookmarkJob, storage::DynStorage};
use tokio::sync::Mutex;

pub async fn run(
    job: ImportBookmarksJob,
    data: Data<Arc<Mutex<DynStorage<ScrapeBookmarkJob>>>>,
) -> Result<(), apalis::prelude::Error> {
    tracing::debug!("Importing {} bookmarks", job.urls.len());

    let mut storage = data.lock().await;

    for url in job.urls {
        storage
            .push(ScrapeBookmarkJob {
                url,
                user_id: job.user_id,
            })
            .await
            .map_err(|e| apalis::prelude::Error::Failed(Arc::new(Box::new(e))))?;
    }

    Ok(())
}
