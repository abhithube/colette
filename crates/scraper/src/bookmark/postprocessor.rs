use anyhow::anyhow;
use chrono::{DateTime, Utc};
use url::Url;

use super::{BookmarkExtractorOptions, ExtractedBookmark};
use crate::PostprocessorError;

#[derive(Clone, Debug, Default)]
pub struct ProcessedBookmark {
    pub title: String,
    pub thumbnail: Option<Url>,
    pub published: Option<DateTime<Utc>>,
    pub author: Option<String>,
}

pub trait BookmarkPostprocessor: Send + Sync {
    fn postprocess(
        &self,
        url: &Url,
        extracted: ExtractedBookmark,
    ) -> Result<ProcessedBookmark, PostprocessorError>;
}

pub type BookmarkPostprocessorFn =
    fn(url: &Url, extracted: ExtractedBookmark) -> Result<ProcessedBookmark, PostprocessorError>;

pub enum BookmarkPostprocessorPlugin<'a> {
    Value(BookmarkExtractorOptions<'a>),
    Callback(BookmarkPostprocessorFn),
}

pub struct DefaultBookmarkPostprocessor;

impl BookmarkPostprocessor for DefaultBookmarkPostprocessor {
    fn postprocess(
        &self,
        _url: &Url,
        mut extracted: ExtractedBookmark,
    ) -> Result<ProcessedBookmark, PostprocessorError> {
        let Some(title) = extracted.title else {
            return Err(PostprocessorError(anyhow!(
                "could not process bookmark title"
            )));
        };

        if let Some(t) = &extracted.thumbnail {
            if t.starts_with("//") {
                extracted.thumbnail = Some(format!("https:{t}"));
            }
        }

        let bookmark = ProcessedBookmark {
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
