use s3::{
    Bucket, Region,
    creds::{Credentials, error::CredentialsError},
    error::S3Error,
};

pub trait S3Client: Sync {
    fn get_object(&self, path: &str) -> impl Future<Output = Result<Vec<u8>, Error>> + Send;

    fn exists_object(&self, path: &str) -> impl Future<Output = Result<bool, Error>> + Send;

    fn put_object(&self, path: &str, data: &[u8])
    -> impl Future<Output = Result<(), Error>> + Send;

    fn delete_object(&self, path: &str) -> impl Future<Output = Result<(), Error>> + Send;
}

#[derive(Clone)]
pub struct S3ClientImpl {
    bucket: Box<Bucket>,
}

impl S3ClientImpl {
    pub async fn init(config: S3Config) -> Result<Self, Error> {
        let region = Region::Custom {
            region: config.region,
            endpoint: config.endpoint,
        };
        let credentials = Credentials::new(
            Some(&config.access_key_id),
            Some(&config.secret_access_key),
            None,
            None,
            None,
        )?;

        let mut bucket = Bucket::new(&config.bucket_name, region, credentials)?;

        if config.path_style_enabled {
            bucket.set_path_style();
        }

        let exists = bucket.exists().await?;
        if !exists {
            return Err(Error::BucketNotFound(config.bucket_name));
        }

        Ok(Self { bucket })
    }
}

impl S3Client for S3ClientImpl {
    async fn get_object(&self, path: &str) -> Result<Vec<u8>, Error> {
        let data = self.bucket.get_object(path).await?;

        Ok(data.to_vec())
    }

    async fn exists_object(&self, path: &str) -> Result<bool, Error> {
        self.bucket.head_object(path).await?;

        Ok(true)
    }

    async fn put_object(&self, path: &str, data: &[u8]) -> Result<(), Error> {
        self.bucket.put_object(path, data).await?;

        Ok(())
    }

    async fn delete_object(&self, path: &str) -> Result<(), Error> {
        self.bucket.delete_object(path).await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct S3Config {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub region: String,
    pub endpoint: String,
    pub bucket_name: String,
    pub path_style_enabled: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bucket not found with name {0}")]
    BucketNotFound(String),

    #[error(transparent)]
    Client(#[from] S3Error),

    #[error(transparent)]
    Auth(#[from] CredentialsError),
}
