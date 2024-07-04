#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] pub anyhow::Error);

pub trait Extractor<T> {
    fn extract(&self, url: &str, raw: &str) -> Result<T, Error>;
}
