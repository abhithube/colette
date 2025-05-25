#[cfg(not(any(feature = "fs", feature = "s3")))]
compile_error!("at least one of 'fs' or 's3' must be enabled");

#[cfg(feature = "fs")]
pub use fs::FsStorageClient;
#[cfg(feature = "s3")]
pub use s3::S3StorageClient;

#[cfg(feature = "fs")]
mod fs;
#[cfg(feature = "s3")]
mod s3;

#[async_trait::async_trait]
pub trait StorageClient: Send + Sync + 'static {
    async fn upload(&self, path: &str, data: Vec<u8>) -> Result<(), std::io::Error>;

    async fn download(&self, path: &str) -> Result<Vec<u8>, std::io::Error>;

    async fn delete(&self, path: &str) -> Result<(), std::io::Error>;

    async fn exists(&self, path: &str) -> Result<bool, std::io::Error>;
}

#[derive(Clone)]
pub enum StorageAdapter {
    #[cfg(feature = "fs")]
    Fs(FsStorageClient),
    #[cfg(feature = "s3")]
    S3(S3StorageClient),
}

#[async_trait::async_trait]
impl StorageClient for StorageAdapter {
    async fn upload(&self, path: &str, data: Vec<u8>) -> Result<(), std::io::Error> {
        match self {
            #[cfg(feature = "fs")]
            StorageAdapter::Fs(storage) => storage.upload(path, data).await,
            #[cfg(feature = "s3")]
            StorageAdapter::S3(storage) => storage.upload(path, data).await,
        }
    }

    async fn download(&self, path: &str) -> Result<Vec<u8>, std::io::Error> {
        match self {
            #[cfg(feature = "fs")]
            StorageAdapter::Fs(storage) => storage.download(path).await,
            #[cfg(feature = "s3")]
            StorageAdapter::S3(storage) => storage.download(path).await,
        }
    }

    async fn delete(&self, path: &str) -> Result<(), std::io::Error> {
        match self {
            #[cfg(feature = "fs")]
            StorageAdapter::Fs(storage) => storage.delete(path).await,
            #[cfg(feature = "s3")]
            StorageAdapter::S3(storage) => storage.delete(path).await,
        }
    }

    async fn exists(&self, path: &str) -> Result<bool, std::io::Error> {
        match self {
            #[cfg(feature = "fs")]
            StorageAdapter::Fs(storage) => storage.exists(path).await,
            #[cfg(feature = "s3")]
            StorageAdapter::S3(storage) => storage.exists(path).await,
        }
    }
}
