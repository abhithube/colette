use bytes::Bytes;
use dyn_clone::DynClone;
use url::Url;

use crate::DownloaderError;
#[cfg(not(target_arch = "wasm32"))]
pub use default::DefaultDownloader;

#[cfg(not(target_arch = "wasm32"))]
mod default;

#[async_trait::async_trait]
pub trait Downloader: Send + Sync + DynClone {
    async fn download(&self, url: &mut Url) -> Result<Bytes, DownloaderError>;
}

dyn_clone::clone_trait_object!(Downloader);
