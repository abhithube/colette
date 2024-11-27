use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::Queue;

#[derive(Clone)]
pub struct InMemoryQueue<Data> {
    sender: Sender<Data>,
}

impl<Data> InMemoryQueue<Data> {
    pub fn new() -> (Self, Receiver<Data>) {
        let (sender, receiver) = mpsc::channel(100);
        (Self { sender }, receiver)
    }
}

#[async_trait::async_trait]
impl<T: Clone + Send + Sync + 'static> Queue for InMemoryQueue<T> {
    type Data = T;

    async fn push(&self, data: Self::Data) -> Result<(), anyhow::Error> {
        self.sender.send(data).await.map_err(|e| e.into())
    }
}
