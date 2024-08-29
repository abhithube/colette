use std::sync::Arc;

use url::Url;

pub trait Postprocessor: Send + Sync {
    type Extracted;
    type Processed;

    fn postprocess(&self, url: &Url, extracted: Self::Extracted) -> Result<Self::Processed, Error>;
}

pub enum PostprocessorPlugin<T, U, V> {
    Value(U),
    Impl(Arc<dyn Postprocessor<Extracted = T, Processed = V>>),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] pub anyhow::Error);
