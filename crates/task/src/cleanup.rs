use std::sync::Arc;

use apalis::prelude::Data;
use chrono::Local;
use colette_core::cleanup::{CleanupJob, CleanupService};
use tracing::{error, info};

pub async fn cleanup(_job: CleanupJob, service: Data<Arc<CleanupService>>) {
    let start = Local::now();
    info!("Started cleanup task");

    match service.cleanup().await {
        Ok(info) => {
            if info.feed_count > 0 {
                info!("Deleted {} orphaned feeds", info.feed_count);
            }
            if info.feed_entry_count > 0 {
                info!("Deleted {} orphaned feed entries", info.feed_entry_count);
            }

            let elasped = (Local::now().time() - start.time()).num_milliseconds();
            info!("Finished cleanup task in {} ms", elasped);
        }
        Err(e) => error!("{:?}", e),
    }
}
