use colette_http::{HttpClient, HyperClient};
use s3::Bucket;
use url::Url;

use crate::Archiver;

const BASE_DIR: &str = "colette";

#[derive(Clone)]
pub struct ThumbnailArchiver {
    client: HyperClient,
    bucket: Box<Bucket>,
}

pub struct ThumbnailData {
    pub url: Url,
    pub file_name: String,
}

impl ThumbnailArchiver {
    pub fn new(http: HyperClient, bucket: Box<Bucket>) -> Self {
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
        let (_, body) = self.client.get(&data.url).await?;

        let format = image::guess_format(&body)?;
        let extension = format.extensions_str()[0];

        let object_path = format!("{}/{}.{}", BASE_DIR, data.file_name, extension);

        self.bucket.put_object(&object_path, &body).await?;

        let url = Url::parse(&format!("{}/{}", self.bucket.url(), object_path)).unwrap();

        Ok(url)
    }
}
