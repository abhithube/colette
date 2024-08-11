use anyhow::anyhow;
use chrono::DateTime;
use colette_core::{
    feeds::{ExtractedFeed, ProcessedEntry, ProcessedFeed},
    scraper::{PostprocessError, Postprocessor},
};
use url::Url;

const RFC2822_WITHOUT_COMMA: &str = "%a %d %b %Y %H:%M:%S %z";

pub struct DefaultFeedPostprocessor {}

impl Postprocessor for DefaultFeedPostprocessor {
    type T = ExtractedFeed;
    type U = ProcessedFeed;

    fn postprocess(
        &self,
        _url: &Url,
        extracted: ExtractedFeed,
    ) -> Result<ProcessedFeed, PostprocessError> {
        let Some(Ok(link)) = extracted.link.as_ref().map(|e| Url::parse(e)) else {
            return Err(PostprocessError(anyhow!("could not process feed link")));
        };
        let Some(title) = extracted.title else {
            return Err(PostprocessError(anyhow!("could not process feed title")));
        };

        let mut entries: Vec<ProcessedEntry> = vec![];

        for e in extracted.entries.into_iter() {
            let Some(Ok(link)) = e.link.as_ref().map(|e| Url::parse(e)) else {
                return Err(PostprocessError(anyhow!("could not process entry link")));
            };
            let Some(title) = e.title else {
                return Err(PostprocessError(anyhow!("could not process entry title")));
            };
            let published = e.published.as_ref().and_then(|e| {
                DateTime::parse_from_rfc3339(e)
                    .ok()
                    .or(DateTime::parse_from_rfc2822(e).ok())
                    .or(DateTime::parse_from_str(e, RFC2822_WITHOUT_COMMA).ok())
                    .map(|f| f.to_utc())
            });
            let thumbnail = e.thumbnail.as_ref().and_then(|e| Url::parse(e).ok());

            let entry = ProcessedEntry {
                link,
                title,
                published,
                description: e.description,
                author: e.author,
                thumbnail,
            };
            entries.push(entry);
        }

        let feed = ProcessedFeed {
            link,
            title,
            entries,
        };

        Ok(feed)
    }
}
