use tokio::sync::mpsc::{self, error::SendError, Receiver, Sender};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait JobProducer: Send + Sync + 'static {
    async fn push(&mut self, job_id: Uuid) -> Result<(), Error>;
}

#[async_trait::async_trait]
pub trait JobConsumer: Send + Sync + 'static {
    async fn pop(&mut self) -> Result<Option<Uuid>, Error>;
}

#[derive(Debug, Clone)]
pub struct TokioJobProducer {
    tx: Sender<Uuid>,
}

#[async_trait::async_trait]
impl JobProducer for TokioJobProducer {
    async fn push(&mut self, job_id: Uuid) -> Result<(), Error> {
        self.tx.send(job_id).await?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct TokioJobConsumer {
    rx: Receiver<Uuid>,
}

#[async_trait::async_trait]
impl JobConsumer for TokioJobConsumer {
    async fn pop(&mut self) -> Result<Option<Uuid>, Error> {
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
    Send(#[from] SendError<Uuid>),
}
