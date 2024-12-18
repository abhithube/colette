use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use colette_core::feed::{self, FeedService};
use colette_queue::Queue;
use futures::StreamExt;
use tower::Service;
use url::Url;

use crate::scrape_feed;

#[derive(Clone)]
pub struct Task {
    service: FeedService,
    scrape_feed_queue: Box<dyn Queue<Data = scrape_feed::Data>>,
}

impl Task {
    pub fn new(
        service: FeedService,
        scrape_feed_queue: Box<dyn Queue<Data = scrape_feed::Data>>,
    ) -> Self {
        Self {
            service,
            scrape_feed_queue,
        }
    }
}

impl Service<()> for Task {
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
