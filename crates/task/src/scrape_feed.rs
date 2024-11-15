use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use colette_core::scraper::{self, FeedCreate, ScraperService};
use tower::Service;
use url::Url;

#[derive(Debug, Clone)]
pub struct Data {
    pub url: Url,
}

#[derive(Clone)]
pub struct Task {
    service: ScraperService,
}

impl Task {
    pub fn new(service: ScraperService) -> Self {
        Self { service }
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
        let service = self.service.clone();

        Box::pin(async move { service.scrape_feed(FeedCreate { url: req.url }).await })
    }
}
