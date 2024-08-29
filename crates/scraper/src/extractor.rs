use std::sync::Arc;

use http::Response;
use url::Url;

pub trait Extractor: Send + Sync {
    type T;

    fn extract(&self, url: &Url, resp: Response<String>) -> Result<Self::T, Error>;
}

pub enum ExtractorPlugin<T, U> {
    Value(T),
    Impl(Arc<dyn Extractor<T = U>>),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] pub anyhow::Error);
