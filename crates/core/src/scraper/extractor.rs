#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] anyhow::Error);

pub trait Extractor<T> {
    fn extract(url: String, raw: String) -> Result<T, Error>;
}
