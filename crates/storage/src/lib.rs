pub use local::LocalStorageClient;

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
    Local(LocalStorageClient),
}

#[async_trait::async_trait]
impl StorageClient for StorageAdapter {
    async fn upload(&self, path: &str, data: Vec<u8>) -> Result<(), std::io::Error> {
        match self {
            StorageAdapter::Local(storage) => storage.upload(path, data).await,
        }
    }

    async fn download(&self, path: &str) -> Result<Vec<u8>, std::io::Error> {
        match self {
            StorageAdapter::Local(storage) => storage.download(path).await,
        }
    }

    async fn delete(&self, path: &str) -> Result<(), std::io::Error> {
        match self {
            StorageAdapter::Local(storage) => storage.delete(path).await,
        }
    }

    async fn exists(&self, path: &str) -> Result<bool, std::io::Error> {
        match self {
            StorageAdapter::Local(storage) => storage.exists(path).await,
        }
    }
}
