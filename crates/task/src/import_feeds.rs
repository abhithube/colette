use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use colette_core::scraper;
use colette_queue::Queue;
use tower::Service;
use url::Url;

use crate::scrape_feed;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Data {
    pub urls: Vec<Url>,
}

#[derive(Clone)]
pub struct Task {
    scrape_feed_queue: Box<dyn Queue<Data = scrape_feed::Data>>,
}

impl Task {
    pub fn new(scrape_feed_queue: Box<dyn Queue<Data = scrape_feed::Data>>) -> Self {
        Self { scrape_feed_queue }
    }
}

impl Service<Data> for Task {
    type Response = ();
    type Error = scraper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Data) -> Self::Future {
        let scrape_feed_queue = self.scrape_feed_queue.clone();

        Box::pin(async move {
            for url in req.urls {
                scrape_feed_queue
                    .push(scrape_feed::Data { url })
                    .await
                    .unwrap();
            }

            Ok(())
        })
    }
}
