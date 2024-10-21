use std::sync::Arc;

use apalis::prelude::{Data, Job};
use colette_core::scraper::{FeedCreate, ScraperService};
use url::Url;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Args {
    pub url: Url,
}

impl Job for Args {
    const NAME: &'static str = "apalis::ScrapeFeed";
}

pub async fn run(args: Args, service: Data<Arc<ScraperService>>) {
    let _ = service.scrape_feed(FeedCreate { url: args.url }).await;
}
