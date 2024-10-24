use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use colette_core::scraper;
use tower::Service;
use url::Url;

use crate::{scrape_bookmark, TaskQueue};

#[derive(Debug, Clone)]
pub struct Data {
    pub urls: Vec<Url>,
}

#[derive(Clone)]
pub struct Task {
    scrape_bookmark_queue: Arc<TaskQueue<scrape_bookmark::Data>>,
}

impl Task {
    pub fn new(scrape_bookmark_queue: Arc<TaskQueue<scrape_bookmark::Data>>) -> Self {
        Self {
            scrape_bookmark_queue,
        }
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
        let scrape_bookmark_queue = self.scrape_bookmark_queue.clone();

        Box::pin(async move {
            for url in req.urls {
                scrape_bookmark_queue
                    .push(scrape_bookmark::Data { url })
                    .await
                    .unwrap();
            }

            Ok(())
        })
    }
}
