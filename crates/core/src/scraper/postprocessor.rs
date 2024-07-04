#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] pub anyhow::Error);

pub trait Postprocessor<T, U> {
    fn postprocess(&self, url: &str, extracted: T) -> Result<U, Error>;
}
