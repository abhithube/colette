use crate::DownloaderError;
use bytes::Bytes;
pub use default::DefaultDownloader;
use dyn_clone::DynClone;
use url::Url;

mod default;

#[async_trait::async_trait]
pub trait Downloader: Send + Sync + DynClone {
    async fn download(&self, url: &mut Url) -> Result<Bytes, DownloaderError>;
}

dyn_clone::clone_trait_object!(Downloader);
