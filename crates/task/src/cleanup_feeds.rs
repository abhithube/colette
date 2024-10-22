use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use colette_core::cleanup::{self, CleanupService};
use tower::Service;
use tracing::info;

#[derive(Clone)]
pub struct Task {
    service: Arc<CleanupService>,
}

impl Task {
    pub fn new(service: Arc<CleanupService>) -> Self {
        Self { service }
    }
}

impl Service<()> for Task {
    type Response = ();
    type Error = cleanup::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: ()) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            let res = service.cleanup().await?;
            if res.feed_count > 0 {
                info!("Deleted {} orphaned feeds", res.feed_count);
            }
            if res.feed_entry_count > 0 {
                info!("Deleted {} orphaned feed entries", res.feed_entry_count);
            }

            Ok(())
        })
    }
}
