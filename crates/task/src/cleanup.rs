use std::sync::Arc;

use apalis::prelude::Data;
use colette_core::cleanup::{CleanupJob, CleanupService};

pub async fn cleanup(_job: CleanupJob, service: Data<Arc<CleanupService>>) {
    if let Err(e) = service.cleanup().await {
        println!("{:?}", e);
    }
}
