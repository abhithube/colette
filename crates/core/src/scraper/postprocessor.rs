#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] pub anyhow::Error);

pub trait Postprocessor<'a, T, U> {
    fn postprocess(&self, url: &str, extracted: &'a T) -> Result<U, Error>;
}
