use core::str;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
};

use bytes::Buf;
use url::Url;

use crate::{Downloader, Error, ExtractorError};

use super::{ExtractedFeed, FeedDetector, FeedScraper, ProcessedFeed};

#[derive(Clone)]
pub struct FeedPluginRegistry {
    plugins: HashMap<&'static str, Box<dyn FeedScraper>>,
    downloader: Box<dyn Downloader>,
    default_scraper: Box<dyn FeedScraper>,
}

impl FeedPluginRegistry {
    pub fn new(
        plugins: HashMap<&'static str, Box<dyn FeedScraper>>,
        downloader: Box<dyn Downloader>,
        default_scraper: Box<dyn FeedScraper>,
    ) -> Self {
        Self {
            plugins,
            downloader,
            default_scraper,
        }
    }
}

#[async_trait::async_trait]
impl FeedScraper for FeedPluginRegistry {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedFeed, Error> {
        let host = url.host_str().ok_or(Error::Parse)?;

        match self.plugins.get(host) {
            Some(plugin) => plugin.scrape(url).await,
            None => self.default_scraper.scrape(url).await,
        }
    }
}

#[async_trait::async_trait]
impl FeedDetector for FeedPluginRegistry {
    async fn detect(&self, mut url: Url) -> Result<Vec<(Url, ProcessedFeed)>, Error> {
        let body = self.downloader.download(&mut url).await?;

        let mut reader = BufReader::new(body.reader());
        let buffer = reader
            .fill_buf()
            .map_err(|e| Error::Extract(ExtractorError(e.into())))?;

        let raw = str::from_utf8(buffer).map_err(|_| Error::Parse)?;

        match raw {
            raw if raw.contains("<!DOCTYPE html") => {
                let metadata = colette_meta::parse_metadata(reader).map_err(|_| Error::Parse)?;

                let mut feeds: Vec<(Url, ProcessedFeed)> = Vec::new();
                for feed in metadata.feeds {
                    let mut url = Url::parse(&feed.href).unwrap();
                    let feed = self.scrape(&mut url).await?;

                    feeds.push((url, feed));
                }

                Ok(feeds)
            }
            raw if raw.contains("<?xml") => {
                let feed = colette_feed::from_reader(BufReader::new(reader))
                    .map(ExtractedFeed::from)
                    .map_err(ExtractorError)?;

                Ok(vec![(url, feed.try_into()?)])
            }
            _ => Err(Error::Parse),
        }
    }
}
