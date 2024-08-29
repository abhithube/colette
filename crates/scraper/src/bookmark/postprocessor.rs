use anyhow::anyhow;
use chrono::{DateTime, Utc};
use url::Url;

use super::ExtractedBookmark;
use crate::postprocessor::{Error, Postprocessor};

#[derive(Clone, Debug, Default)]
pub struct ProcessedBookmark {
    pub title: String,
    pub thumbnail: Option<Url>,
    pub published: Option<DateTime<Utc>>,
    pub author: Option<String>,
}

pub struct DefaultBookmarkPostprocessor {}

impl Postprocessor for DefaultBookmarkPostprocessor {
    type Extracted = ExtractedBookmark;
    type Processed = ProcessedBookmark;

    fn postprocess(
        &self,
        _url: &Url,
        mut extracted: ExtractedBookmark,
    ) -> Result<ProcessedBookmark, Error> {
        let Some(title) = extracted.title else {
            return Err(Error(anyhow!("could not process bookmark title")));
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
