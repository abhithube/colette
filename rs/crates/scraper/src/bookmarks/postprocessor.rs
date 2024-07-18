use anyhow::anyhow;
use chrono::DateTime;
use colette_core::{
    bookmarks::{ExtractedBookmark, ProcessedBookmark},
    utils::scraper::{PostprocessError, Postprocessor},
};
use url::Url;

pub struct DefaultBookmarkPostprocessor {}

impl Postprocessor<ExtractedBookmark, ProcessedBookmark> for DefaultBookmarkPostprocessor {
    fn postprocess(
        &self,
        _url: &str,
        extracted: ExtractedBookmark,
    ) -> Result<ProcessedBookmark, PostprocessError> {
        let Some(Ok(link)) = extracted.link.as_ref().map(|e| Url::parse(e)) else {
            return Err(PostprocessError(anyhow!("could not process bookmark link")));
        };
        let Some(title) = extracted.title else {
            return Err(PostprocessError(anyhow!(
                "could not process bookmark title"
            )));
        };

        let bookmark = ProcessedBookmark {
            link,
            title,
            thumbnail: extracted
                .thumbnail
                .as_ref()
                .and_then(|e| Url::parse(e).ok()),
            published: extracted.published.as_ref().and_then(|e| {
                DateTime::parse_from_rfc3339(e)
                    .ok()
                    .or(DateTime::parse_from_rfc2822(e).ok())
                    .map(|f| f.to_utc())
            }),
            author: extracted.author,
        };

        Ok(bookmark)
    }
}
