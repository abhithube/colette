use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use colette_core::feed::{self, FeedService};
use colette_queue::Queue;
use futures::StreamExt;
use tower::Service;
use url::Url;

use crate::scrape_feed;

#[derive(Clone)]
pub struct Task<Q> {
    service: Arc<FeedService>,
    scrape_feed_queue: Q,
}

impl<Q: Queue<Data = scrape_feed::Data>> Task<Q> {
    pub fn new(service: Arc<FeedService>, scrape_feed_queue: Q) -> Self {
        Self {
            service,
            scrape_feed_queue,
        }
    }
}

impl<Q: Queue<Data = scrape_feed::Data> + Clone> Service<()> for Task<Q> {
    type Response = ();
    type Error = feed::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: ()) -> Self::Future {
        let service = self.service.clone();
        let scrape_feed_queue = self.scrape_feed_queue.clone();

        Box::pin(async move {
            let mut stream = service.stream().await?;

            while let Some(raw) = stream.next().await {
                let url = Url::parse(&raw).unwrap();

                scrape_feed_queue
                    .push(scrape_feed::Data { url })
                    .await
                    .unwrap()
            }

            Ok(())
        })
    }
}
