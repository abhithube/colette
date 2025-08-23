use chrono::{DateTime, Utc};
use colette_util::uuid_generate_ts;
use serde::Serialize;
use serde_json::Value;
use tokio::sync::mpsc::{self, error::SendError, Receiver, Sender};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Job {
    pub id: Uuid,
    pub job_type: String,
    pub data: Value,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Job {
    #[allow(clippy::result_large_err)]
    pub fn create<T: Into<String>, D: Serialize>(job_type: T, data: D) -> Result<Self, Error> {
        let now = Utc::now();

        let data = serde_json::to_value(data)?;

        Ok(Self {
            id: uuid_generate_ts(now),
            job_type: job_type.into(),
            data,
            message: None,
            created_at: now,
        })
    }
}

#[async_trait::async_trait]
pub trait JobProducer: Send + Sync + 'static {
    async fn push(&mut self, job: Job) -> Result<(), Error>;
}

#[async_trait::async_trait]
pub trait JobConsumer: Send + Sync + 'static {
    async fn pop(&mut self) -> Result<Option<Job>, Error>;
}

#[derive(Debug, Clone)]
pub struct TokioJobProducer {
    tx: Sender<Job>,
}

#[async_trait::async_trait]
impl JobProducer for TokioJobProducer {
    async fn push(&mut self, job: Job) -> Result<(), Error> {
        self.tx.send(job).await?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct TokioJobConsumer {
    rx: Receiver<Job>,
}

#[async_trait::async_trait]
impl JobConsumer for TokioJobConsumer {
    async fn pop(&mut self) -> Result<Option<Job>, Error> {
        let next = self.rx.recv().await;

        Ok(next)
    }
}

#[derive(Debug)]
pub struct TokioQueue {
    producer: TokioJobProducer,
    consumer: TokioJobConsumer,
}

impl TokioQueue {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);

        Self {
            producer: TokioJobProducer { tx },
            consumer: TokioJobConsumer { rx },
        }
    }

    pub fn split(self) -> (TokioJobProducer, TokioJobConsumer) {
        (self.producer, self.consumer)
    }

    pub fn producer(&mut self) -> &mut TokioJobProducer {
        &mut self.producer
    }

    pub fn consumer(&mut self) -> &mut TokioJobConsumer {
        &mut self.consumer
    }
}

impl Default for TokioQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Send(#[from] SendError<Job>),

    #[error(transparent)]
    Serialize(#[from] serde_json::Error),
}
