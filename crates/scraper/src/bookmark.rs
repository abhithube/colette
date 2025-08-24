use std::{collections::HashMap, io::BufReader, str::Utf8Error};

use bytes::Buf;
use chrono::{DateTime, Utc};
use colette_http::HttpClient;
use colette_meta::{
    Metadata,
    basic::Basic,
    open_graph::{self, OpenGraph},
    schema_org::{
        Article, ImageObject, Person, SchemaObject, SchemaObjectOrValue, SocialMediaPosting,
        TypeOrString, VideoObject, WebPage, WebSite,
    },
};
use url::Url;

const RFC3339_WITH_MILLI: &str = "%Y-%m-%dT%H:%M:%S%.3f%z";
const RFC3339_WITH_MICRO: &str = "%Y-%m-%dT%H:%M:%S%.6f%z";

#[async_trait::async_trait]
pub trait BookmarkPlugin: Send + Sync {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedBookmark, BookmarkError>;
}

pub struct BookmarkScraper<HC: HttpClient> {
    client: HC,
    plugins: HashMap<&'static str, Box<dyn BookmarkPlugin>>,
}

impl<HC: HttpClient> BookmarkScraper<HC> {
    pub fn new(client: HC, plugins: HashMap<&'static str, Box<dyn BookmarkPlugin>>) -> Self {
        Self { client, plugins }
    }

    pub async fn scrape(&self, url: &mut Url) -> Result<ProcessedBookmark, BookmarkError> {
        let host = url.host_str().unwrap();

        match self.plugins.get(host) {
            Some(plugin) => plugin.scrape(url).await,
            None => {
                let body = self.client.get(url).await?;
                let mut reader = BufReader::new(body.reader());

                let raw = str::from_utf8(reader.peek(14)?)?;
                if !raw.to_lowercase().contains("<!doctype html") {
                    return Err(BookmarkError::Unsupported);
                }

                let extracted = colette_meta::parse_metadata(reader)
                    .map(ExtractedBookmark::from)
                    .map_err(BookmarkError::Parse)?;

                extracted.try_into().map_err(BookmarkError::Postprocess)
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExtractedBookmark {
    pub title: Option<String>,
    pub thumbnail: Option<String>,
    pub published: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ProcessedBookmark {
    pub title: String,
    pub thumbnail: Option<Url>,
    pub published: Option<DateTime<Utc>>,
    pub author: Option<String>,
}

impl TryFrom<ExtractedBookmark> for ProcessedBookmark {
    type Error = PostprocessorError;

    fn try_from(mut value: ExtractedBookmark) -> Result<Self, Self::Error> {
        let Some(title) = value.title else {
            return Err(PostprocessorError::Title);
        };

        if let Some(ref t) = value.thumbnail
            && t.starts_with("//")
        {
            value.thumbnail = Some(format!("https:{t}"));
        }

        let bookmark = ProcessedBookmark {
            title,
            thumbnail: value.thumbnail.as_ref().and_then(|e| Url::parse(e).ok()),
            published: value.published.as_ref().and_then(|e| {
                DateTime::parse_from_rfc3339(e)
                    .ok()
                    .or(DateTime::parse_from_rfc2822(e).ok())
                    .or(DateTime::parse_from_str(e, RFC3339_WITH_MILLI).ok())
                    .or(DateTime::parse_from_str(e, RFC3339_WITH_MICRO).ok())
                    .map(|f| f.to_utc())
            }),
            author: value.author,
        };

        Ok(bookmark)
    }
}

impl From<Metadata> for ExtractedBookmark {
    fn from(value: Metadata) -> Self {
        let mut bookmark = Self::from(value.basic);
        if let Some(open_graph) = value.open_graph {
            let og_bookmark = Self::from(open_graph);
            bookmark.title = og_bookmark.title;
            bookmark.thumbnail = og_bookmark.thumbnail;
            bookmark.published = og_bookmark.published;
        }

        if bookmark.title.is_none()
            || bookmark.thumbnail.is_none()
            || bookmark.published.is_none()
            || bookmark.author.is_none()
        {
            for schema in value.schema_org {
                if let SchemaObjectOrValue::SchemaObject(schema) = schema {
                    let new = Self::from(schema);

                    bookmark.title = bookmark.title.or(new.title);
                    bookmark.thumbnail = bookmark.thumbnail.or(new.thumbnail);
                    bookmark.published = bookmark.published.or(new.published);
                    bookmark.author = bookmark.author.or(new.author);
                }
            }
        }

        bookmark
    }
}

impl From<Basic> for ExtractedBookmark {
    fn from(value: Basic) -> Self {
        Self {
            title: value.title,
            author: value.author,
            ..Default::default()
        }
    }
}

impl From<OpenGraph> for ExtractedBookmark {
    fn from(mut value: OpenGraph) -> Self {
        Self {
            title: Some(value.title),
            thumbnail: if !value.images.is_empty() {
                Some(value.images.swap_remove(0).url)
            } else {
                None
            },
            published: if let open_graph::Type::Article(article) = value.r#type {
                article.published_time
            } else {
                None
            },
            ..Default::default()
        }
    }
}

impl From<Article> for ExtractedBookmark {
    fn from(value: Article) -> Self {
        let mut video: Option<ExtractedBookmark> = value.video.map(Into::into);

        Self {
            title: value
                .headline
                .or(value.name)
                .or(video.as_mut().and_then(|e| e.title.take())),
            thumbnail: value
                .thumbnail_url
                .or(value.thumbnail.and_then(|e| e.url))
                .or(video.as_mut().and_then(|e| e.thumbnail.take())),
            published: value
                .date_published
                .or(video.as_mut().and_then(|e| e.published.take())),
            author: value
                .author
                .and_then(|e| match e {
                    TypeOrString::Type(person) => person.name,
                    TypeOrString::String(raw) => Some(raw),
                })
                .or(video.as_mut().and_then(|e| e.author.take())),
        }
    }
}

impl From<SocialMediaPosting> for ExtractedBookmark {
    fn from(value: SocialMediaPosting) -> Self {
        let mut video = value.video.map(ExtractedBookmark::from);

        Self {
            title: value
                .headline
                .or(value.name)
                .or(video.as_mut().and_then(|e| e.title.take())),
            thumbnail: value
                .thumbnail_url
                .or(value.thumbnail.and_then(|e| e.url))
                .or(video.as_mut().and_then(|e| e.thumbnail.take())),
            published: value
                .date_published
                .or(video.as_mut().and_then(|e| e.published.take())),
            author: value
                .author
                .and_then(|e| match e {
                    TypeOrString::Type(person) => person.name,
                    TypeOrString::String(raw) => Some(raw),
                })
                .or(video.as_mut().and_then(|e| e.author.take())),
        }
    }
}

impl From<WebPage> for ExtractedBookmark {
    fn from(value: WebPage) -> Self {
        let mut video = value.video.map(ExtractedBookmark::from);

        Self {
            title: value
                .headline
                .or(value.name)
                .or(video.as_mut().and_then(|e| e.title.take())),
            thumbnail: value
                .thumbnail_url
                .or(value.thumbnail.and_then(|e| e.url))
                .or(video.as_mut().and_then(|e| e.thumbnail.take())),
            published: value
                .date_published
                .or(video.as_mut().and_then(|e| e.published.take())),
            author: value
                .author
                .and_then(|e| match e {
                    TypeOrString::Type(person) => person.name,
                    TypeOrString::String(raw) => Some(raw),
                })
                .or(video.as_mut().and_then(|e| e.author.take())),
        }
    }
}

impl From<VideoObject> for ExtractedBookmark {
    fn from(value: VideoObject) -> Self {
        Self {
            title: value.headline.or(value.name),
            thumbnail: value.thumbnail_url.or(value.thumbnail.and_then(|e| e.url)),
            published: value.date_published,
            author: value.author.and_then(|e| match e {
                TypeOrString::Type(person) => person.name,
                TypeOrString::String(raw) => Some(raw),
            }),
        }
    }
}

impl From<WebSite> for ExtractedBookmark {
    fn from(value: WebSite) -> Self {
        Self {
            title: value.headline.or(value.name),
            thumbnail: value.thumbnail_url.or(value.thumbnail.and_then(|e| e.url)),
            published: value.date_published,
            author: value.author.and_then(|e| e.name),
        }
    }
}

impl From<ImageObject> for ExtractedBookmark {
    fn from(value: ImageObject) -> Self {
        Self {
            thumbnail: value.url,
            ..Default::default()
        }
    }
}

impl From<Person> for ExtractedBookmark {
    fn from(value: Person) -> Self {
        Self {
            author: value.name,
            ..Default::default()
        }
    }
}

impl From<SchemaObject> for ExtractedBookmark {
    fn from(value: SchemaObject) -> Self {
        match value {
            SchemaObject::Article(article) => article.into(),
            SchemaObject::SocialMediaPosting(smp) => smp.into(),
            SchemaObject::WebPage(webpage) => webpage.into(),
            SchemaObject::VideoObject(video) => video.into(),
            SchemaObject::WebSite(website) => website.into(),
            SchemaObject::ImageObject(image) => image.into(),
            SchemaObject::Person(person) => person.into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BookmarkError {
    #[error("document type not supported")]
    Unsupported,

    #[error(transparent)]
    Parse(#[from] colette_meta::Error),

    #[error(transparent)]
    Postprocess(#[from] PostprocessorError),

    #[error(transparent)]
    Http(#[from] colette_http::Error),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Utf(#[from] Utf8Error),
}

#[derive(Debug, thiserror::Error)]
pub enum PostprocessorError {
    #[error("could not process link")]
    Link,

    #[error("could not process title")]
    Title,

    #[error("could not process published date")]
    Published,
}
