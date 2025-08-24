pub use command::*;
pub use query::*;

mod command;
mod query;

#[async_trait::async_trait]
pub trait Handler<C> {
    type Response;
    type Error: std::error::Error;

    async fn handle(&self, cmd: C) -> Result<Self::Response, Self::Error>;
}
