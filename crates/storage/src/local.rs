use std::path::PathBuf;

use tokio::{fs::File, io::AsyncReadExt};

use crate::StorageClient;

#[derive(Clone)]
pub struct LocalStorageClient {
    path: PathBuf,
}

impl LocalStorageClient {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

#[async_trait::async_trait]
impl StorageClient for LocalStorageClient {
    async fn upload(&self, path: &str, data: Vec<u8>) -> Result<(), std::io::Error> {
        tokio::fs::create_dir_all(&self.path).await?;
        tokio::fs::write(self.path.join(path), data).await?;

        Ok(())
    }

    async fn download(&self, path: &str) -> Result<Vec<u8>, std::io::Error> {
        let mut file = File::open(self.path.join(path)).await?;

        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await?;

        Ok(buf)
    }

    async fn delete(&self, path: &str) -> Result<(), std::io::Error> {
        tokio::fs::remove_file(self.path.join(path)).await?;

        Ok(())
    }

    async fn exists(&self, path: &str) -> Result<bool, std::io::Error> {
        tokio::fs::try_exists(self.path.join(path)).await
    }
}
