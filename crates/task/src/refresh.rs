use std::sync::Arc;

use apalis::prelude::Data;
use colette_core::refresh::{RefreshJob, RefreshService};

pub async fn refresh_feeds(_job: RefreshJob, service: Data<Arc<RefreshService>>) {
    if let Err(e) = service.refresh_feeds().await {
        println!("{:?}", e);
    }
}
