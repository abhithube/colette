use std::sync::Arc;

use apalis::prelude::Data;
use chrono::Local;
use colette_core::refresh::{RefreshJob, RefreshService};

pub async fn refresh_feeds(_job: RefreshJob, service: Data<Arc<RefreshService>>) {
    let start = Local::now();
    println!("Started refresh task at: {}", start.to_rfc3339());

    match service.refresh_feeds().await {
        Ok(_) => {
            let elasped = (Local::now().time() - start.time()).num_milliseconds();
            println!("Finished refresh task in {} ms", elasped);
        }
        Err(e) => println!("{:?}", e),
    }
}
