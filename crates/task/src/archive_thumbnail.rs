use std::sync::Arc;

use apalis::prelude::Data;
use colette_core::bookmark::{BookmarkService, ThumbnailArchive};
use url::Url;
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Job {
    pub url: Url,
    pub bookmark_id: Uuid,
    pub user_id: Uuid,
}

pub async fn run(job: Job, data: Data<Arc<BookmarkService>>) -> Result<(), apalis::prelude::Error> {
    tracing::debug!("Archiving thumbnail URL: {}", job.url.as_str());

    data.archive_thumbnail(
        job.bookmark_id,
        ThumbnailArchive {
            thumbnail_url: job.url,
        },
        job.user_id,
    )
    .await
    .map_err(|e| apalis::prelude::Error::Failed(Arc::new(Box::new(e))))?;

    Ok(())
}
