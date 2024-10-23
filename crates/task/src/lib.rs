use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use chrono::Utc;
use cron::Schedule;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tower::{Service, ServiceExt};
use tracing::error;

pub mod cleanup_feeds;
pub mod import_feeds;
pub mod refresh_feeds;
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

pub async fn run_task_worker<Data: Send + 'static, S: Service<Data> + Clone + Send + 'static>(
    mut receiver: Receiver<Data>,
    task: S,
) where
    S::Error: Debug + Display,
    S::Future: Send,
{
    while let Some(data) = receiver.recv().await {
        let mut task = task.clone();

        tokio::spawn(async move {
            if let Err(e) = task.ready().await.unwrap().call(data).await {
                error!("{}", e);
            }
        });
    }
}

pub async fn run_cron_worker<S: Service<()> + Clone + Send + 'static>(cron: &str, mut task: S)
where
    S::Error: Debug + Display,
    S::Future: Send,
{
    let schedule = Schedule::from_str(cron).unwrap();

    loop {
        let upcoming = schedule.upcoming(Utc).take(1).next().unwrap();
        let duration = (upcoming - Utc::now()).to_std().unwrap();

        tokio::time::sleep(duration).await;

        if let Err(e) = task.ready().await.unwrap().call(()).await {
            error!("{}", e);
        }
    }
}
