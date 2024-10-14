use std::sync::Arc;

use apalis::prelude::Data;
use chrono::Local;
use colette_core::refresh::{RefreshJob, RefreshService};
use tracing::{error, info};

pub async fn refresh_feeds(_job: RefreshJob, service: Data<Arc<RefreshService>>) {
    let start = Local::now();
    info!("Started refresh task");

    match service.refresh_feeds().await {
        Ok(_) => {
            let elasped = (Local::now().time() - start.time()).num_milliseconds();
            info!("Finished refresh task in {} ms", elasped);
        }
        Err(e) => error!("{:?}", e),
    }
}
