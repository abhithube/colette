use apalis::prelude::Storage as ApalisStorage;
use apalis_core::request::Parts;

pub mod import_bookmarks;
pub mod import_feeds;
pub mod refresh_feeds;
pub mod scrape_bookmark;
pub mod scrape_feed;

#[async_trait::async_trait]
pub trait Storage: Send {
    type Job;

    async fn push(&mut self, job: Self::Job) -> Result<Parts<()>, Error>;
}

#[async_trait::async_trait]
impl<T: ApalisStorage + Send> Storage for T
where
    T::Job: Send,
    T::Error: Into<anyhow::Error>,
{
    type Job = T::Job;

    async fn push(&mut self, job: Self::Job) -> Result<Parts<()>, Error> {
        self.push(job)
            .await
            .map(|p| {
                let mut parts = Parts::default();
                parts.attempt = p.attempt;
                parts.data = p.data;
                parts.namespace = p.namespace;
                parts.task_id = p.task_id;
                parts.context = ();

                parts
            })
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
