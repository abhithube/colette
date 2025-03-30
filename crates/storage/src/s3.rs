use std::io::ErrorKind;

use s3::Bucket;

use crate::StorageClient;

#[derive(Clone)]
pub struct S3StorageClient {
    bucket: Box<Bucket>,
}

impl S3StorageClient {
    pub fn new(bucket: Box<Bucket>) -> Self {
        Self { bucket }
    }
}

#[async_trait::async_trait]
impl StorageClient for S3StorageClient {
    async fn upload(&self, path: &str, data: Vec<u8>) -> Result<(), std::io::Error> {
        self.bucket
            .put_object(path, &data)
            .await
            .map_err(|e| std::io::Error::new(ErrorKind::Other, e.to_string()))?;

        Ok(())
    }

    async fn download(&self, path: &str) -> Result<Vec<u8>, std::io::Error> {
        let data = self
            .bucket
            .get_object(path)
            .await
            .map_err(|e| std::io::Error::new(ErrorKind::Other, e.to_string()))?;

        Ok(data.to_vec())
    }

    async fn delete(&self, path: &str) -> Result<(), std::io::Error> {
        self.bucket
            .delete_object(path)
            .await
            .map_err(|e| std::io::Error::new(ErrorKind::Other, e.to_string()))?;

        Ok(())
    }

    async fn exists(&self, path: &str) -> Result<bool, std::io::Error> {
        self.bucket
            .object_exists(path)
            .await
            .map_err(|e| std::io::Error::new(ErrorKind::Other, e.to_string()))
    }
}
