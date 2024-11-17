use tokio::sync::mpsc::{self, Receiver, Sender};

pub mod cleanup_feeds;
pub mod import_bookmarks;
pub mod import_feeds;
pub mod refresh_feeds;
pub mod scrape_bookmark;
pub mod scrape_feed;

#[derive(Clone)]
pub struct TaskQueue<Data> {
    sender: Sender<Data>,
}

impl<Data> TaskQueue<Data> {
    pub fn new() -> (Self, Receiver<Data>) {
        let (sender, receiver) = mpsc::channel(100);
        (Self { sender }, receiver)
    }

    pub async fn push(&self, data: Data) -> Result<(), mpsc::error::SendError<Data>> {
        self.sender.send(data).await
    }
}
