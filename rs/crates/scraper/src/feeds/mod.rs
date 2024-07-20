use std::sync::Arc;

use anyhow::anyhow;
use atom::AtomFeed;
use colette_core::{
    feeds::{ExtractedFeed, ProcessedFeed},
    utils::scraper::{Downloader, Error, ExtractError, PluginRegistry, Postprocessor, Scraper},
};
pub use extractor::{HtmlExtractor, Item, Node, TextSelector};
pub use postprocessor::DefaultFeedPostprocessor;
use rss::RSSFeed;
use url::Url;

use crate::DefaultDownloader;

mod atom;
mod extractor;
mod postprocessor;
mod rss;

pub struct FeedScraper {
    registry: PluginRegistry<ExtractedFeed, ProcessedFeed>,
    default_downloader: Arc<dyn Downloader>,
    default_postprocessor: Arc<dyn Postprocessor<ExtractedFeed, ProcessedFeed>>,
}

impl FeedScraper {
    pub fn new(registry: PluginRegistry<ExtractedFeed, ProcessedFeed>) -> Self {
        Self {
            registry,
            default_downloader: Arc::new(DefaultDownloader {}),
            default_postprocessor: Arc::new(DefaultFeedPostprocessor {}),
        }
    }
}

#[async_trait::async_trait]
impl Scraper<ProcessedFeed> for FeedScraper {
    async fn scrape(&self, url: &mut String) -> Result<ProcessedFeed, Error> {
        let parsed = Url::parse(url).map_err(|_| Error::Parse)?;
        let host = parsed.host_str().ok_or(Error::Parse)?;

        let downloader = self
            .registry
            .downloaders
            .get(host)
            .unwrap_or(&self.default_downloader);
        let postprocessor = self
            .registry
            .postprocessors
            .get(host)
            .unwrap_or(&self.default_postprocessor);

        let resp = downloader.download(url).await?;
        let body = String::from_utf8_lossy(resp.body());

        let extracted = match &body {
            raw if raw.contains("<feed") => quick_xml::de::from_str::<AtomFeed>(raw)
                .map(ExtractedFeed::from)
                .map_err(|e| Error::Extract(ExtractError(e.into()))),
            raw if raw.contains("<rss") => quick_xml::de::from_str::<RSSFeed>(raw)
                .map(ExtractedFeed::from)
                .map_err(|e| Error::Extract(ExtractError(e.into()))),
            raw if raw.contains("<html") => {
                if let Some(extractor) = self.registry.extractors.get(host) {
                    extractor
                        .extract(url, raw)
                        .map_err(|e| Error::Extract(ExtractError(e.into())))
                } else {
                    Err(Error::Extract(ExtractError(anyhow!(
                        "couldn't find extractor for feed URL"
                    ))))
                }
            }
            _ => Err(Error::Parse),
        }?;

        let processed = postprocessor.postprocess(url, extracted)?;

        Ok(processed)
    }
}
