use std::sync::Arc;

use url::Url;

pub trait Postprocessor: Send + Sync {
    type T;
    type U;

    fn postprocess(&self, url: &Url, extracted: Self::T) -> Result<Self::U, Error>;
}

pub enum PostprocessorPlugin<T, U, V> {
    Value(U),
    Impl(Arc<dyn Postprocessor<T = T, U = V>>),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] pub anyhow::Error);
