use core::str;
use std::io::Read;

use html5gum::{IoReader, Token, Tokenizer};
use schema_org::{
    Article, ImageObject, Person, SchemaObject, SchemaObjectOrValue, VideoObject, WebPage, WebSite,
};

use crate::{basic::Basic, open_graph::OpenGraph, rss::Feed, util::Value};

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

    in_json_ld: bool,
    current_itemprop: Option<String>,
    schema_stack: Vec<SchemaObjectOrValue>,
}

pub fn parse_metadata<R: Read>(reader: R) -> Metadata {
    let mut metadata = Metadata::default();

    let mut tokenizer = Tokenizer::new(IoReader::new(reader));

    while let Some(Ok(token)) = tokenizer.next() {
        match token {
            Token::StartTag(mut tag) => {
                if let Some(name) = tag.attributes.remove("name".as_bytes()) {
                    if let Some(content) = tag.attributes.remove("content".as_bytes()) {
                        let name = str::from_utf8(&name).unwrap();
                        let content = String::from_utf8(content.0).unwrap();

                        metadata.handle_basic(name, content);
                    }
                } else if let Some(property) = tag.attributes.remove("property".as_bytes()) {
                    if let Some(content) = tag.attributes.remove("content".as_bytes()) {
                        if property.as_slice().starts_with(b"og:") {
                            let property = str::from_utf8(&property).unwrap();
                            let content = String::from_utf8(content.0).unwrap();

                            metadata.handle_open_graph(property, content);
                        }
                    }
                } else if let Some(r#type) = tag.attributes.remove("type".as_bytes()) {
                    if tag.name.as_slice() == b"link" && r#type.as_slice() == b"application/rss+xml"
                    {
                        if let (Some(title), Some(href)) = (
                            tag.attributes.remove("title".as_bytes()),
                            tag.attributes.remove("href".as_bytes()),
                        ) {
                            let title = String::from_utf8(title.0).unwrap();
                            let href = String::from_utf8(href.0).unwrap();

                            metadata.handle_rss(title, href);
                        }
                    } else if r#type.as_slice() == b"application/ld+json" {
                        metadata.in_json_ld = true;
                    }
                } else if let Some(itemtype) = tag.attributes.remove("itemtype".as_bytes()) {
                    let schema = match str::from_utf8(itemtype.as_slice())
                        .unwrap()
                        .split("/")
                        .last()
                    {
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

                    metadata.schema_stack.push(schema);
                } else if let Some(itemprop) = tag.attributes.remove("itemprop".as_bytes()) {
                    if let Some(content) = tag.attributes.remove("content".as_bytes()) {
                        let itemprop = String::from_utf8(itemprop.0).unwrap();
                        let content = String::from_utf8(content.0).unwrap();

                        metadata.current_itemprop = Some(itemprop.clone());
                        metadata.handle_microdata(itemprop, content);
                    }
                }
            }
            Token::String(text) => {
                if metadata.in_json_ld {
                    metadata.handle_json_ld(String::from_utf8(text.0).unwrap());
                }
            }
            Token::EndTag(_) => {
                if metadata.in_json_ld {
                    metadata.in_json_ld = false;
                    continue;
                }

                let Some(completed_schema) = metadata.schema_stack.pop() else {
                    continue;
                };
                let Some(parent_schema) = metadata.schema_stack.last_mut() else {
                    // println!("{:?}", completed_schema);
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
                            match metadata.current_itemprop.as_deref() {
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
                            (other, metadata.current_itemprop.as_deref())
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

    metadata
}
