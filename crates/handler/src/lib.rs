pub use command::*;
pub use common::*;
pub use query::*;
pub use repository::*;

mod command;
mod common;
mod query;
mod repository;

pub trait Handler<C> {
    type Response;
    type Error: std::error::Error;

    fn handle(&self, cmd: C) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send;
}
