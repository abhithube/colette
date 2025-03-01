use core::fmt;
use std::sync::Arc;

use futures::stream::BoxStream;
use object_store::{
    GetOptions, GetResult, ListResult, MultipartUpload, ObjectMeta, ObjectStore, PutMultipartOpts,
    PutOptions, PutPayload, PutResult, aws::AmazonS3, local::LocalFileSystem, path::Path,
};

#[derive(Debug, Clone)]
pub enum StorageAdapter {
    Local(Arc<LocalFileSystem>),
    S3(AmazonS3),
}

impl fmt::Display for StorageAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let raw = match self {
            Self::Local(store) => store.to_string(),
            Self::S3(store) => store.to_string(),
        };

        write!(f, "{}", raw)
    }
}

#[async_trait::async_trait]
impl ObjectStore for StorageAdapter {
    async fn put_opts(
        &self,
        location: &Path,
        payload: PutPayload,
        opts: PutOptions,
    ) -> object_store::Result<PutResult> {
        match self {
            Self::Local(store) => store.put_opts(location, payload, opts).await,
            Self::S3(store) => store.put_opts(location, payload, opts).await,
        }
    }

    async fn put_multipart_opts(
        &self,
        location: &Path,
        opts: PutMultipartOpts,
    ) -> object_store::Result<Box<dyn MultipartUpload>> {
        match self {
            Self::Local(store) => store.put_multipart_opts(location, opts).await,
            Self::S3(store) => store.put_multipart_opts(location, opts).await,
        }
    }

    async fn get_opts(
        &self,
        location: &Path,
        options: GetOptions,
    ) -> object_store::Result<GetResult> {
        match self {
            Self::Local(store) => store.get_opts(location, options).await,
            Self::S3(store) => store.get_opts(location, options).await,
        }
    }

    async fn delete(&self, location: &Path) -> object_store::Result<()> {
        match self {
            Self::Local(store) => store.delete(location).await,
            Self::S3(store) => store.delete(location).await,
        }
    }

    fn list(&self, prefix: Option<&Path>) -> BoxStream<'_, object_store::Result<ObjectMeta>> {
        match self {
            Self::Local(store) => store.list(prefix),
            Self::S3(store) => store.list(prefix),
        }
    }

    async fn list_with_delimiter(&self, prefix: Option<&Path>) -> object_store::Result<ListResult> {
        match self {
            Self::Local(store) => store.list_with_delimiter(prefix).await,
            Self::S3(store) => store.list_with_delimiter(prefix).await,
        }
    }

    async fn copy(&self, from: &Path, to: &Path) -> object_store::Result<()> {
        match self {
            Self::Local(store) => store.copy(from, to).await,
            Self::S3(store) => store.copy(from, to).await,
        }
    }

    async fn copy_if_not_exists(&self, from: &Path, to: &Path) -> object_store::Result<()> {
        match self {
            Self::Local(store) => store.copy_if_not_exists(from, to).await,
            Self::S3(store) => store.copy_if_not_exists(from, to).await,
        }
    }
}
