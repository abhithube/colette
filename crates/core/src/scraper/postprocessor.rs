#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] anyhow::Error);

pub trait Postprocessor<T, U> {
    fn postprocess(url: String, extracted: T) -> Result<U, Error>;
}
