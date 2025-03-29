#[cfg(not(any(feature = "local")))]
panic!("at least one of 'local' must be enabled");

#[cfg(feature = "local")]
pub use local::LocalStorageClient;

#[cfg(feature = "local")]
mod local;

#[async_trait::async_trait]
pub trait StorageClient: Send + Sync + 'static {
    async fn upload(&self, path: &str, data: Vec<u8>) -> Result<(), std::io::Error>;

    async fn download(&self, path: &str) -> Result<Vec<u8>, std::io::Error>;

    async fn delete(&self, path: &str) -> Result<(), std::io::Error>;

    async fn exists(&self, path: &str) -> Result<bool, std::io::Error>;
}

#[derive(Clone)]
pub enum StorageAdapter {
    #[cfg(feature = "local")]
    Local(LocalStorageClient),
}

#[async_trait::async_trait]
impl StorageClient for StorageAdapter {
    async fn upload(&self, path: &str, data: Vec<u8>) -> Result<(), std::io::Error> {
        match self {
            #[cfg(feature = "local")]
            StorageAdapter::Local(storage) => storage.upload(path, data).await,
        }
    }

    async fn download(&self, path: &str) -> Result<Vec<u8>, std::io::Error> {
        match self {
            #[cfg(feature = "local")]
            StorageAdapter::Local(storage) => storage.download(path).await,
        }
    }

    async fn delete(&self, path: &str) -> Result<(), std::io::Error> {
        match self {
            #[cfg(feature = "local")]
            StorageAdapter::Local(storage) => storage.delete(path).await,
        }
    }

    async fn exists(&self, path: &str) -> Result<bool, std::io::Error> {
        match self {
            #[cfg(feature = "local")]
            StorageAdapter::Local(storage) => storage.exists(path).await,
        }
    }
}
