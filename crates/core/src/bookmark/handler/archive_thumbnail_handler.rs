use chrono::Utc;
use colette_http::HttpClient;
use colette_s3::S3Client;
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

pub struct ArchiveThumbnailHandler<BR: BookmarkRepository, HC: HttpClient, SC: S3Client> {
    bookmark_repository: BR,
    http_client: HC,
    s3_client: SC,
}

impl<BR: BookmarkRepository, HC: HttpClient, SC: S3Client> ArchiveThumbnailHandler<BR, HC, SC> {
    pub fn new(bookmark_repository: BR, http_client: HC, s3_client: SC) -> Self {
        Self {
            bookmark_repository,
            http_client,
            s3_client,
        }
    }
}

#[async_trait::async_trait]
impl<BR: BookmarkRepository, HC: HttpClient, SC: S3Client> Handler<ArchiveThumbnailCommand>
    for ArchiveThumbnailHandler<BR, HC, SC>
{
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

                let data: Vec<u8> = body.into();
                self.s3_client.put_object(&object_path, &data).await?;

                self.bookmark_repository
                    .set_archived_path(cmd.bookmark_id, Some(object_path))
                    .await?;
            }
            ThumbnailOperation::Delete => {}
        }

        if let Some(archived_path) = cmd.archived_path {
            self.s3_client.delete_object(&archived_path).await?;

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
    S3(#[from] colette_s3::Error),

    #[error(transparent)]
    Http(#[from] colette_http::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
