use std::{
    any::type_name,
    fmt::{Debug, Display},
    str::FromStr,
};

use chrono::Utc;
use cron::Schedule;
use tokio::sync::mpsc::Receiver;
use tower::Service;
use tower::ServiceExt;
use tracing::{error, info};

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
