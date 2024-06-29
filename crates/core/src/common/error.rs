use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown app error")]
    Unknown,
}
