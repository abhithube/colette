use colette_core::queue::{Error, JobConsumer, JobProducer};
use tokio::sync::mpsc::{self, Receiver, Sender};
use uuid::Uuid;

#[derive(Debug)]
pub struct LocalQueue {
    producer: LocalJobProducer,
    consumer: LocalJobConsumer,
}

impl LocalQueue {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);

        Self {
            producer: LocalJobProducer { tx },
            consumer: LocalJobConsumer { rx },
        }
    }

    pub fn split(self) -> (LocalJobProducer, LocalJobConsumer) {
        (self.producer, self.consumer)
    }

    pub fn producer(&mut self) -> &mut LocalJobProducer {
        &mut self.producer
    }

    pub fn consumer(&mut self) -> &mut LocalJobConsumer {
        &mut self.consumer
    }
}

impl Default for LocalQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct LocalJobProducer {
    tx: Sender<Uuid>,
}

#[async_trait::async_trait]
impl JobProducer for LocalJobProducer {
    async fn push(&mut self, job_id: Uuid) -> Result<(), Error> {
        self.tx
            .send(job_id)
            .await
            .map_err(|e| Error::Backend(e.to_string()))
    }
}

#[derive(Debug)]
pub struct LocalJobConsumer {
    rx: Receiver<Uuid>,
}

#[async_trait::async_trait]
impl JobConsumer for LocalJobConsumer {
    async fn pop(&mut self) -> Result<Option<Uuid>, Error> {
        let next = self.rx.recv().await;

        Ok(next)
    }
}
