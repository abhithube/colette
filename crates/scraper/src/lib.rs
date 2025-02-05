use std::str::Utf8Error;

pub mod bookmark;
pub mod feed;
pub mod utils;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("document type not supported")]
    Unsupported,

    #[error(transparent)]
    Parse(#[from] ParseError),

    #[error(transparent)]
    Postprocess(#[from] PostprocessorError),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Utf(#[from] Utf8Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error(transparent)]
    Feed(#[from] colette_feed::Error),

    #[error(transparent)]
    Meta(#[from] colette_meta::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum PostprocessorError {
    #[error("could not process link")]
    Link,

    #[error("could not process title")]
    Title,

    #[error("could not process published date")]
    Published,
}
