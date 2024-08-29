use std::{collections::HashMap, sync::Arc};

pub use extractor::{BookmarkExtractorOptions, DefaultBookmarkExtractor, ExtractedBookmark};
pub use options::{
    base_extractor_options, microdata_extractor_options, open_graph_extractor_options,
    twitter_extractor_options,
};
pub use postprocessor::{DefaultBookmarkPostprocessor, ProcessedBookmark};
use url::Url;

use crate::{
    downloader::{DefaultDownloader, Downloader, DownloaderPlugin},
    extractor::{Extractor, ExtractorPlugin},
    postprocessor::{Postprocessor, PostprocessorPlugin},
    Scraper,
};

mod extractor;
mod options;
mod postprocessor;

#[derive(Default)]
pub struct BookmarkPluginRegistry<'a> {
    pub downloaders: HashMap<&'static str, DownloaderPlugin<()>>,
    pub extractors:
        HashMap<&'static str, ExtractorPlugin<BookmarkExtractorOptions<'a>, ExtractedBookmark>>,
    pub postprocessors:
        HashMap<&'static str, PostprocessorPlugin<ExtractedBookmark, (), ProcessedBookmark>>,
}

pub struct DefaultBookmarkScraper<'a> {
    registry: BookmarkPluginRegistry<'a>,
    default_downloader: Arc<dyn Downloader>,
    default_extractor: Arc<dyn Extractor<T = ExtractedBookmark>>,
    default_postprocessor: Arc<dyn Postprocessor<T = ExtractedBookmark, U = ProcessedBookmark>>,
}

impl<'a> DefaultBookmarkScraper<'a> {
    pub fn new(registry: BookmarkPluginRegistry<'a>) -> Self {
        Self {
            registry,
            default_downloader: Arc::new(DefaultDownloader {}),
            default_extractor: Arc::new(DefaultBookmarkExtractor::new(None)),
            default_postprocessor: Arc::new(DefaultBookmarkPostprocessor {}),
        }
    }
}

impl Scraper<ProcessedBookmark> for DefaultBookmarkScraper<'_> {
    fn scrape(&self, url: &mut Url) -> Result<ProcessedBookmark, crate::Error> {
        let host = url.host_str().ok_or(crate::Error::Parse)?;

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
