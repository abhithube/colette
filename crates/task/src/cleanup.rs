use std::sync::Arc;

use apalis::prelude::Data;
use chrono::Local;
use colette_core::cleanup::{CleanupJob, CleanupService};

pub async fn cleanup(_job: CleanupJob, service: Data<Arc<CleanupService>>) {
    let start = Local::now();
    println!("Started cleanup task at: {}", start);

    match service.cleanup().await {
        Ok(info) => {
            if info.feed_count > 0 {
                println!("Deleted {} orphaned feeds", info.feed_count);
            }
            if info.feed_entry_count > 0 {
                println!("Deleted {} orphaned feed entries", info.feed_entry_count);
            }

            let elasped = (Local::now().time() - start.time()).num_milliseconds();
            println!("Finished cleanup task in {} ms", elasped);
        }
        Err(e) => println!("{:?}", e),
    }
}
