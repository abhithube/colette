use anyhow::anyhow;
use chrono::DateTime;
use colette_core::{
    feeds::{ExtractedFeed, ProcessedEntry, ProcessedFeed},
    scraper::postprocessor::{Error, Postprocessor},
};
use url::Url;

pub struct DefaultFeedPostprocessor {}

impl<'a> Postprocessor<'a, ExtractedFeed, ProcessedFeed<'a>> for DefaultFeedPostprocessor {
    fn postprocess(_url: String, extracted: &'a ExtractedFeed) -> Result<ProcessedFeed<'a>, Error> {
        let Some(Ok(link)) = extracted.link.as_ref().map(|e| Url::parse(e)) else {
            return Err(Error(anyhow!("could not process feed link")));
        };
        let Some(title) = extracted.title.as_ref() else {
            return Err(Error(anyhow!("could not process feed title")));
        };

        let entries: &mut [ProcessedEntry] = &mut [];

        for (i, e) in extracted.entries.iter().enumerate() {
            let Some(Ok(link)) = e.link.as_ref().map(|e| Url::parse(e)) else {
                return Err(Error(anyhow!("could not process entry link")));
            };
            let Some(title) = e.title.as_ref() else {
                return Err(Error(anyhow!("could not process entry title")));
            };
            let published = e
                .published
                .as_ref()
                .and_then(|e| DateTime::parse_from_rfc3339(e).ok().map(|f| f.to_utc()));
            let thumbnail = e.thumbnail.as_ref().and_then(|e| Url::parse(e).ok());

            let entry = ProcessedEntry {
                link,
                title,
                published,
                description: e.description.as_deref(),
                author: e.author.as_deref(),
                thumbnail,
            };
            entries[i] = entry
        }

        let feed = ProcessedFeed {
            link,
            title,
            entries,
        };

        Ok(feed)
    }
}
