use chrono::Utc;
use colette_http::HttpClient;
use colette_storage::StorageClient;
use colette_util::{hex_encode, sha256_hash};

use crate::{
    Handler,
    bookmark::{BookmarkId, BookmarkRepository, ThumbnailOperation},
    common::RepositoryError,
};

const THUMBNAILS_DIR: &str = "thumbnails";

#[derive(Debug, Clone)]
pub struct ArchiveThumbnailCommand {
    pub bookmark_id: BookmarkId,
    pub operation: ThumbnailOperation,
    pub archived_path: Option<String>,
}

pub struct ArchiveThumbnailHandler {
    bookmark_repository: Box<dyn BookmarkRepository>,
    http_client: Box<dyn HttpClient>,
    storage_client: Box<dyn StorageClient>,
}

impl ArchiveThumbnailHandler {
    pub fn new(
        bookmark_repository: impl BookmarkRepository,
        http_client: impl HttpClient,
        storage_client: impl StorageClient,
    ) -> Self {
        Self {
            bookmark_repository: Box::new(bookmark_repository),
            http_client: Box::new(http_client),
            storage_client: Box::new(storage_client),
        }
    }
}

#[async_trait::async_trait]
impl Handler<ArchiveThumbnailCommand> for ArchiveThumbnailHandler {
    type Response = ();
    type Error = ArchiveThumbnailError;

    async fn handle(&self, cmd: ArchiveThumbnailCommand) -> Result<Self::Response, Self::Error> {
        match cmd.operation {
            ThumbnailOperation::Upload(thumbnail_url) => {
                let file_name = format!(
                    "{}-{}",
                    Utc::now().timestamp(),
                    hex_encode(&sha256_hash(thumbnail_url.as_str())[..8])
                );

                let body = self.http_client.get(&thumbnail_url).await?;

                let format = image::guess_format(&body)?;
                let extension = format.extensions_str()[0];

                let object_path = format!("{THUMBNAILS_DIR}/{file_name}.{extension}");

                self.storage_client
                    .upload(&object_path, body.into())
                    .await?;

                self.bookmark_repository
                    .set_archived_path(cmd.bookmark_id, Some(object_path))
                    .await?;
            }
            ThumbnailOperation::Delete => {}
        }

        if let Some(archived_path) = cmd.archived_path {
            self.storage_client.delete(&archived_path).await?;

            self.bookmark_repository
                .set_archived_path(cmd.bookmark_id, None)
                .await?;
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ArchiveThumbnailError {
    #[error(transparent)]
    Image(#[from] image::ImageError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Http(#[from] colette_http::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
