use std::sync::Arc;

use http::Response;
use url::Url;

pub trait Extractor: Send + Sync {
    type Extracted;

    fn extract(&self, url: &Url, resp: Response<String>) -> Result<Self::Extracted, Error>;
}

pub enum ExtractorPlugin<T, U> {
    Value(T),
    Impl(Arc<dyn Extractor<Extracted = U>>),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] pub anyhow::Error);
