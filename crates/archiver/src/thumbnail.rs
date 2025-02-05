use reqwest::Client;
use s3::Bucket;
use url::Url;

use crate::Archiver;

const BASE_DIR: &str = "colette";

#[derive(Clone)]
pub struct ThumbnailArchiver {
    client: Client,
    bucket: Box<Bucket>,
}

pub struct ThumbnailData {
    pub url: Url,
    pub file_name: String,
}

impl ThumbnailArchiver {
    pub fn new(http: Client, bucket: Box<Bucket>) -> Self {
        Self {
            client: http,
            bucket,
        }
    }
}

#[async_trait::async_trait]
impl Archiver<ThumbnailData> for ThumbnailArchiver {
    type Output = Url;

    async fn archive(&self, data: ThumbnailData) -> Result<Self::Output, crate::Error> {
        let resp = self.client.get(data.url.as_str()).send().await?;

        let raw = resp.bytes().await?;

        let format = image::guess_format(&raw)?;
        let extension = format.extensions_str()[0];

        let object_path = format!("{}/{}.{}", BASE_DIR, data.file_name, extension);

        self.bucket.put_object(&object_path, &raw).await?;

        let url = Url::parse(&format!("{}/{}", self.bucket.url(), object_path)).unwrap();

        Ok(url)
    }
}
