use core::str;
use std::{io::Read, str::Utf8Error};

use html5gum::{IoReader, Token, Tokenizer};
use open_graph::handle_open_graph;
use rss::{Feed, handle_rss};
use schema_org::{
    Article, ImageObject, Person, SchemaObject, SchemaObjectOrValue, VideoObject, WebPage, WebSite,
    handle_json_ld, handle_microdata,
};

use crate::{basic::Basic, open_graph::OpenGraph, util::Value};

pub mod basic;
pub mod open_graph;
pub mod rss;
pub mod schema_org;
pub mod util;

#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub basic: Basic,
    pub feeds: Vec<Feed>,
    pub open_graph: Option<OpenGraph>,
    pub schema_org: Vec<SchemaObjectOrValue>,
}

pub fn parse_metadata<R: Read>(reader: R) -> Result<Metadata, Error> {
    let mut metadata = Metadata::default();

    let mut in_json_ld = false;
    let mut current_itemprop: Option<String> = None;
    let mut schema_stack: Vec<SchemaObjectOrValue> = Vec::new();

    let mut tokenizer = Tokenizer::new(IoReader::new(reader));

    while let Some(Ok(token)) = tokenizer.next() {
        match token {
            Token::StartTag(mut tag) => {
                if let Some(name) = tag.attributes.remove("name".as_bytes()) {
                    if let Some(content) = tag.attributes.remove("content".as_bytes()) {
                        let name = str::from_utf8(&name).map_err(ParseError::Utf)?;
                        let content = String::from_utf8(content.0)
                            .map_err(|e| e.utf8_error())
                            .map_err(ParseError::Utf)?;

                        metadata.handle_basic(name, content);
                    }
                } else if let Some(property) = tag.attributes.remove("property".as_bytes()) {
                    if let Some(content) = tag.attributes.remove("content".as_bytes()) {
                        if property.as_slice().starts_with(b"og:") {
                            let property = str::from_utf8(&property).map_err(ParseError::Utf)?;
                            let content = String::from_utf8(content.0)
                                .map_err(|e| e.utf8_error())
                                .map_err(ParseError::Utf)?;

                            let open_graph =
                                metadata.open_graph.get_or_insert_with(OpenGraph::default);
                            handle_open_graph(open_graph, property, content);
                        }
                    }
                } else if let Some(r#type) = tag.attributes.remove("type".as_bytes()) {
                    if tag.name.as_slice() == b"link" && r#type.as_slice() == b"application/rss+xml"
                    {
                        if let (Some(title), Some(href)) = (
                            tag.attributes.remove("title".as_bytes()),
                            tag.attributes.remove("href".as_bytes()),
                        ) {
                            let title = String::from_utf8(title.0)
                                .map_err(|e| e.utf8_error())
                                .map_err(ParseError::Utf)?;
                            let href = String::from_utf8(href.0)
                                .map_err(|e| e.utf8_error())
                                .map_err(ParseError::Utf)?;

                            handle_rss(&mut metadata.feeds, title, href);
                        }
                    } else if r#type.as_slice() == b"application/ld+json" {
                        in_json_ld = true;
                    }
                } else if let Some(itemtype) = tag.attributes.remove("itemtype".as_bytes()) {
                    let url = str::from_utf8(itemtype.as_slice()).map_err(ParseError::Utf)?;
                    let schema = match url.split("/").last() {
                        Some("Article") => SchemaObjectOrValue::SchemaObject(
                            SchemaObject::Article(Article::default()),
                        ),
                        Some("WebPage") => SchemaObjectOrValue::SchemaObject(
                            SchemaObject::WebPage(WebPage::default()),
                        ),
                        Some("VideoObject") => SchemaObjectOrValue::SchemaObject(
                            SchemaObject::VideoObject(VideoObject::default()),
                        ),
                        Some("WebSite") => SchemaObjectOrValue::SchemaObject(
                            SchemaObject::WebSite(WebSite::default()),
                        ),
                        Some("ImageObject") => SchemaObjectOrValue::SchemaObject(
                            SchemaObject::ImageObject(ImageObject::default()),
                        ),
                        Some("Person") => SchemaObjectOrValue::SchemaObject(SchemaObject::Person(
                            Person::default(),
                        )),
                        _ => SchemaObjectOrValue::Other(Value::default()),
                    };

                    schema_stack.push(schema);
                } else if let Some(itemprop) = tag.attributes.remove("itemprop".as_bytes()) {
                    if let Some(content) = tag.attributes.remove("content".as_bytes()) {
                        let itemprop = String::from_utf8(itemprop.0)
                            .map_err(|e| e.utf8_error())
                            .map_err(ParseError::Utf)?;
                        let content = String::from_utf8(content.0)
                            .map_err(|e| e.utf8_error())
                            .map_err(ParseError::Utf)?;

                        current_itemprop = Some(itemprop.clone());
                        if let Some(schema_org) = schema_stack.last_mut() {
                            handle_microdata(schema_org, itemprop, content);
                        }
                    }
                }
            }
            Token::String(text) => {
                if in_json_ld {
                    let json_ld = String::from_utf8(text.0)
                        .map_err(|e| e.utf8_error())
                        .map_err(ParseError::Utf)?;

                    handle_json_ld(&mut metadata.schema_org, json_ld);
                }
            }
            Token::EndTag(_) => {
                if in_json_ld {
                    in_json_ld = false;
                    continue;
                }

                let Some(completed_schema) = schema_stack.pop() else {
                    continue;
                };
                let Some(parent_schema) = schema_stack.last_mut() else {
                    metadata.schema_org.push(completed_schema);
                    continue;
                };

                let mut author: Option<Person> = None;
                let mut image: Option<ImageObject> = None;
                let mut thumbnail: Option<ImageObject> = None;
                let mut value: Option<Value> = None;

                match completed_schema {
                    SchemaObjectOrValue::SchemaObject(schema) => match schema {
                        SchemaObject::ImageObject(image_object) => {
                            match current_itemprop.as_deref() {
                                Some("image") => image = Some(image_object),
                                Some("thumbnail") => thumbnail = Some(image_object),
                                _ => {}
                            }
                        }
                        SchemaObject::Person(person) => {
                            author = Some(person);
                        }
                        _ => {}
                    },
                    SchemaObjectOrValue::Other(other) => {
                        value = Some(other);
                    }
                }

                match parent_schema {
                    SchemaObjectOrValue::SchemaObject(schema) => match schema {
                        SchemaObject::Article(article) => {
                            if author.is_some() {
                                article.author = author;
                            }
                            if image.is_some() {
                                article.image = image;
                            }
                            if thumbnail.is_some() {
                                article.thumbnail = thumbnail;
                            }
                        }
                        SchemaObject::WebPage(webpage) => {
                            if author.is_some() {
                                webpage.author = author;
                            }
                            if image.is_some() {
                                webpage.image = image;
                            }
                            if thumbnail.is_some() {
                                webpage.thumbnail = thumbnail;
                            }
                        }
                        SchemaObject::VideoObject(video_object) => {
                            if author.is_some() {
                                video_object.author = author;
                            }
                            if image.is_some() {
                                video_object.image = image;
                            }
                            if thumbnail.is_some() {
                                video_object.thumbnail = thumbnail;
                            }
                        }
                        SchemaObject::WebSite(website) => {
                            if author.is_some() {
                                website.author = author;
                            }
                            if image.is_some() {
                                website.image = image;
                            }
                            if thumbnail.is_some() {
                                website.thumbnail = thumbnail;
                            }
                        }
                        SchemaObject::ImageObject(image_object) => {
                            if author.is_some() {
                                image_object.author = author.map(Box::new);
                            }
                            if image.is_some() {
                                image_object.image = image.map(Box::new);
                            }
                            if thumbnail.is_some() {
                                image_object.thumbnail = thumbnail.map(Box::new);
                            }
                        }
                        SchemaObject::Person(person) => {
                            if image.is_some() {
                                person.image = image;
                            }
                        }
                    },
                    SchemaObjectOrValue::Other(other) => {
                        if let (Value::Object(object), Some(itemprop)) =
                            (other, current_itemprop.as_deref())
                        {
                            let other = object
                                .entry(itemprop.to_owned())
                                .or_insert(Value::Array(Vec::new()));
                            if let (Some(value), Value::Array(array)) = (value, other) {
                                array.push(value)
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(metadata)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Parse(#[from] ParseError),
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error(transparent)]
    Utf(#[from] Utf8Error),
}
