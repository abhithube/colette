use apalis_core::{request::Parts, storage::Storage as ApalisStorage};

#[async_trait::async_trait]
pub trait Storage: Send {
    type Job;
    type Error;
    type Context: Default;

    async fn push(&mut self, job: Self::Job) -> Result<Parts<Self::Context>, Self::Error>;
}

#[async_trait::async_trait]
impl<T: ApalisStorage + Send> Storage for T
where
    T::Job: Send,
{
    type Job = T::Job;
    type Error = T::Error;
    type Context = T::Context;

    async fn push(&mut self, job: Self::Job) -> Result<Parts<Self::Context>, Self::Error> {
        self.push(job).await
    }
}
