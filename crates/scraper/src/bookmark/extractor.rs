use http::Response;
use scraper::Html;
use url::Url;

use crate::{
    base_extractor_options,
    extractor::{Error, Extractor},
    microdata_extractor_options, open_graph_extractor_options, twitter_extractor_options,
    utils::{ExtractorQuery, TextSelector},
};

#[derive(Clone, Debug, Default)]
pub struct BookmarkExtractorOptions<'a> {
    pub title_queries: Vec<ExtractorQuery<'a>>,
    pub published_queries: Vec<ExtractorQuery<'a>>,
    pub author_queries: Vec<ExtractorQuery<'a>>,
    pub thumbnail_queries: Vec<ExtractorQuery<'a>>,
}

#[derive(Clone, Debug, Default)]
pub struct ExtractedBookmark {
    pub title: Option<String>,
    pub thumbnail: Option<String>,
    pub published: Option<String>,
    pub author: Option<String>,
}

pub struct DefaultBookmarkExtractor<'a> {
    options: BookmarkExtractorOptions<'a>,
}

impl<'a> DefaultBookmarkExtractor<'a> {
    pub fn new(options: Option<BookmarkExtractorOptions<'a>>) -> Self {
        Self {
            options: options.unwrap_or(merge(vec![
                open_graph_extractor_options(),
                twitter_extractor_options(),
                microdata_extractor_options(),
                base_extractor_options(),
            ])),
        }
    }
}

impl Extractor for DefaultBookmarkExtractor<'_> {
    type Extracted = ExtractedBookmark;

    fn extract(&self, _url: &Url, resp: Response<String>) -> Result<ExtractedBookmark, Error> {
        let raw = resp.into_body();
        let html = Html::parse_document(&raw);

        let bookmark = ExtractedBookmark {
            title: html.select_text(&self.options.title_queries),
            thumbnail: html.select_text(&self.options.thumbnail_queries),
            published: html.select_text(&self.options.published_queries),
            author: html.select_text(&self.options.author_queries),
        };

        Ok(bookmark)
    }
}

fn merge(options_vec: Vec<BookmarkExtractorOptions>) -> BookmarkExtractorOptions {
    fn merge_field<'a>(fields: &[Vec<ExtractorQuery<'a>>]) -> Vec<ExtractorQuery<'a>> {
        fields.iter().flat_map(|v| v.iter().cloned()).collect()
    }

    BookmarkExtractorOptions {
        title_queries: merge_field(
            &options_vec
                .iter()
                .map(|e| e.title_queries.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
        thumbnail_queries: merge_field(
            &options_vec
                .iter()
                .map(|e| e.thumbnail_queries.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
        published_queries: merge_field(
            &options_vec
                .iter()
                .map(|e| e.published_queries.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
        author_queries: merge_field(
            &options_vec
                .iter()
                .map(|e| e.author_queries.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
    }
}
