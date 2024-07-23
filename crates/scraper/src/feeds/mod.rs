use std::sync::Arc;

use colette_core::{
    feeds::{ExtractedFeed, FeedExtractorOptions, ProcessedFeed},
    utils::scraper::{
        Downloader, DownloaderPlugin, Error, Extractor, ExtractorPlugin, PluginRegistry,
        Postprocessor, PostprocessorPlugin, Scraper,
    },
};
use extractor::DefaultFeedExtractor;
pub use extractor::{HtmlExtractor, TextSelector};
pub use postprocessor::DefaultFeedPostprocessor;
use url::Url;

use crate::DefaultDownloader;

mod atom;
mod extractor;
mod postprocessor;
mod rss;

pub struct FeedScraper<'a> {
    registry: PluginRegistry<FeedExtractorOptions<'a>, ExtractedFeed, (), ProcessedFeed>,
    default_downloader: Arc<dyn Downloader>,
    default_extractor: Arc<dyn Extractor<T = ExtractedFeed>>,
    default_postprocessor: Arc<dyn Postprocessor<T = ExtractedFeed, U = ProcessedFeed>>,
}

impl<'a> FeedScraper<'a> {
    pub fn new(
        registry: PluginRegistry<FeedExtractorOptions<'a>, ExtractedFeed, (), ProcessedFeed>,
    ) -> Self {
        Self {
            registry,
            default_downloader: Arc::new(DefaultDownloader {}),
            default_extractor: Arc::new(DefaultFeedExtractor {}),
            default_postprocessor: Arc::new(DefaultFeedPostprocessor {}),
        }
    }
}

impl Scraper<ProcessedFeed> for FeedScraper<'_> {
    fn scrape(&self, url: &mut String) -> Result<ProcessedFeed, Error> {
        let parsed = Url::parse(url).map_err(|_| Error::Parse)?;
        let host = parsed.host_str().ok_or(Error::Parse)?;

        let downloader = self.registry.downloaders.get(host);
        let extractor = self.registry.extractors.get(host);
        let postprocessor = self.registry.postprocessors.get(host);

        let resp = match downloader {
            Some(DownloaderPlugin::Impl(downloader)) => downloader.download(url),
            _ => self.default_downloader.download(url),
        }?;

        let extracted = match extractor {
            Some(ExtractorPlugin::Impl(extractor)) => extractor.extract(url, resp),
            _ => self.default_extractor.extract(url, resp),
        }?;

        let processed = match postprocessor {
            Some(PostprocessorPlugin::Impl(postprocessor)) => {
                postprocessor.postprocess(url, extracted)
            }
            _ => self.default_postprocessor.postprocess(url, extracted),
        }?;

        Ok(processed)
    }
}
