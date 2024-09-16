use std::sync::Arc;

use apalis::prelude::Data;
use chrono::{DateTime, Local, Utc};
use colette_core::refresh::RefreshService;

#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub struct RefreshJob(DateTime<Utc>);
impl From<DateTime<Utc>> for RefreshJob {
    fn from(t: DateTime<Utc>) -> Self {
        RefreshJob(t)
    }
}

pub async fn refresh_feeds(_job: RefreshJob, service: Data<Arc<RefreshService>>) {
    let start = Local::now();
    println!("Started refresh task at: {}", start.to_rfc3339());

    match service.refresh_feeds().await {
        Ok(_) => {
            let elasped = (Local::now().time() - start.time()).num_milliseconds();
            println!("Finished refresh task in {} ms", elasped);
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
}
