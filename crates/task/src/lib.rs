use std::{
    any::type_name,
    fmt::{Debug, Display},
    str::FromStr,
};

use chrono::Utc;
use cron::Schedule;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tower::{Service, ServiceExt};
use tracing::{error, info};

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

pub async fn run_task_worker<Data: Send + 'static, S: Service<Data> + Clone + Send + 'static>(
    mut receiver: Receiver<Data>,
    task: S,
) where
    S::Error: Debug + Display,
    S::Future: Send,
{
    let name = type_name::<S>();

    while let Some(data) = receiver.recv().await {
        let mut task = task.clone();

        tokio::spawn(async move {
            info!("Started task {}", name);

            match task.ready().await.unwrap().call(data).await {
                Ok(_) => {
                    info!("Finished task {}", name);
                }
                Err(e) => {
                    error!("{}", e);
                }
            }
        });
    }
}

pub async fn run_cron_worker<S: Service<()> + Clone + Send + 'static>(cron: &str, mut task: S)
where
    S::Error: Debug + Display,
    S::Future: Send,
{
    let name = type_name::<S>();

    let schedule = Schedule::from_str(cron).unwrap();

    loop {
        let upcoming = schedule.upcoming(Utc).take(1).next().unwrap();
        let duration = (upcoming - Utc::now()).to_std().unwrap();

        tokio::time::sleep(duration).await;

        info!("Started task {}", name);

        match task.ready().await.unwrap().call(()).await {
            Ok(_) => {
                info!("Finished task {}", name);
            }
            Err(e) => {
                error!("{}", e);
            }
        }
    }
}
