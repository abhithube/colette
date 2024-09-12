use std::{
    cell::{Cell, RefCell},
    io::{BufRead, BufReader, Read},
};

use basic::{handle_basic, Basic};
use html5ever::{
    tendril::StrTendril,
    tokenizer::{
        BufferQueue, Tag, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts,
    },
};
use open_graph::{handle_open_graph, OpenGraph};
use rss::{handle_rss, Feed};
use schema_org::{
    handle_json_ld, handle_microdata, Article, ImageObject, Person, SchemaObject,
    SchemaObjectOrValue, VideoObject, WebPage, WebSite,
};
use util::Value;

pub mod basic;
pub mod open_graph;
pub mod rss;
pub mod schema_org;
pub mod util;

#[derive(Debug, Clone, Default)]
struct MetadataSink {
    basic: RefCell<Basic>,
    feeds: RefCell<Vec<Feed>>,
    open_graph: RefCell<Option<OpenGraph>>,
    schema_org: RefCell<Vec<SchemaObjectOrValue>>,
    in_ld_json: Cell<bool>,
    inner_text: RefCell<StrTendril>,
    current_itemprop: RefCell<Option<StrTendril>>,
    schema_stack: RefCell<Vec<SchemaObjectOrValue>>,
}

impl TokenSink for MetadataSink {
    type Handle = ();

    fn process_token(&self, token: Token, _line_number: u64) -> TokenSinkResult<Self::Handle> {
        match token {
            Token::CharacterTokens(inner_text) => self.handle_inner_text(inner_text),
            Token::TagToken(tag) => match tag.kind {
                TagKind::StartTag => self.handle_start_tag(tag),
                TagKind::EndTag => self.handle_end_tag(tag),
            },
            _ => {}
        }

        TokenSinkResult::Continue
    }
}

impl MetadataSink {
    fn handle_start_tag(&self, tag: Tag) {
        let mut content: Option<StrTendril> = None;
        let mut itemprop: Option<StrTendril> = None;
        let mut itemtype: Option<StrTendril> = None;
        let mut href: Option<StrTendril> = None;
        let mut name: Option<StrTendril> = None;
        let mut property: Option<StrTendril> = None;
        let mut title: Option<StrTendril> = None;
        let mut r#type: Option<StrTendril> = None;

        for attr in tag.attrs {
            match attr.name.local.as_ref() {
                "content" => content = Some(attr.value),
                "itemprop" => itemprop = Some(attr.value),
                "itemtype" => itemtype = Some(attr.value),
                "href" => href = Some(attr.value),
                "name" => name = Some(attr.value),
                "property" => property = Some(attr.value),
                "title" => title = Some(attr.value),
                "type" => r#type = Some(attr.value),
                _ => {}
            }
        }

        match (content, itemprop, itemtype, href, name, property, title) {
            (Some(content), _, _, _, Some(name), _, _) if tag.name.as_ref() == "meta" => {
                let mut basic = self.basic.borrow_mut();
                handle_basic(&mut basic, name.into(), content.into());
            }
            (_, _, _, Some(href), _, _, Some(title))
                if tag.name.as_ref() == "link"
                    && r#type.as_deref() == Some("application/rss+xml") =>
            {
                handle_rss(&mut self.feeds.borrow_mut(), title.into(), href.into());
            }
            (Some(content), _, _, _, _, Some(mut property), _) if property.contains(":") => {
                let mut open_graph = self.open_graph.borrow_mut();
                let open_graph = open_graph.get_or_insert_with(OpenGraph::default);

                if let Some((_, suffix)) = property.split_once(':') {
                    property = suffix.into();
                }

                handle_open_graph(open_graph, property.into(), content.into());
            }
            _ if tag.name.as_ref() == "script"
                && r#type.as_deref() == Some("application/ld+json") =>
            {
                self.in_ld_json.set(true);
            }
            (_, _, Some(itemtype), _, _, _, _) => {
                let schema = match itemtype.split("/").last() {
                    Some("Article") => {
                        SchemaObjectOrValue::SchemaObject(SchemaObject::Article(Article::default()))
                    }
                    Some("WebPage") => {
                        SchemaObjectOrValue::SchemaObject(SchemaObject::WebPage(WebPage::default()))
                    }
                    Some("VideoObject") => SchemaObjectOrValue::SchemaObject(
                        SchemaObject::VideoObject(VideoObject::default()),
                    ),
                    Some("WebSite") => {
                        SchemaObjectOrValue::SchemaObject(SchemaObject::WebSite(WebSite::default()))
                    }
                    Some("ImageObject") => SchemaObjectOrValue::SchemaObject(
                        SchemaObject::ImageObject(ImageObject::default()),
                    ),
                    Some("Person") => {
                        SchemaObjectOrValue::SchemaObject(SchemaObject::Person(Person::default()))
                    }
                    _ => SchemaObjectOrValue::Other(Value::default()),
                };

                let mut stack = self.schema_stack.borrow_mut();
                stack.push(schema);
            }
            (content, Some(itemprop), _, href, _, _, _) => {
                if let Some(current) = self.schema_stack.borrow_mut().last_mut() {
                    self.current_itemprop.replace(Some(itemprop.clone()));

                    if let Some(content) = content.or(href) {
                        handle_microdata(current, itemprop.into(), content.into());
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_inner_text(&self, inner_text: StrTendril) {
        if self.in_ld_json.get() {
            let mut ld_json = self.inner_text.borrow_mut();
            ld_json.push_tendril(&inner_text);
        }
    }

    fn handle_end_tag(&self, tag: Tag) {
        if tag.name.as_ref() == "script" && self.in_ld_json.get() {
            let text = self.inner_text.take();
            let mut schema_org = self.schema_org.borrow_mut();

            handle_json_ld(&mut schema_org, text.into());

            self.in_ld_json.set(false);

            return;
        }

        let mut stack = self.schema_stack.borrow_mut();
        let itemprop = self.current_itemprop.take();
        if let Some(completed_schema) = stack.pop() {
            if let Some(parent_schema) = stack.last_mut() {
                let mut author: Option<Person> = None;
                let mut image: Option<ImageObject> = None;
                let mut thumbnail: Option<ImageObject> = None;
                let mut value: Option<Value> = None;

                match completed_schema {
                    SchemaObjectOrValue::SchemaObject(schema) => match schema {
                        SchemaObject::ImageObject(image_object) => match itemprop.as_deref() {
                            Some("image") => image = Some(image_object),
                            Some("thumbnail") => thumbnail = Some(image_object),
                            _ => {}
                        },
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
                        if let (Value::Object(object), Some(itemprop)) = (other, itemprop) {
                            let other = object
                                .entry(itemprop.into())
                                .or_insert(Value::Array(Vec::new()));
                            if let (Some(value), Value::Array(array)) = (value, other) {
                                array.push(value)
                            }
                        }
                    }
                }
            } else {
                self.schema_org.borrow_mut().push(completed_schema);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub basic: Basic,
    pub feeds: Vec<Feed>,
    pub open_graph: Option<OpenGraph>,
    pub schema_org: Vec<SchemaObjectOrValue>,
}

pub fn parse_metadata<R: Read>(reader: R) -> Metadata {
    let reader = BufReader::new(reader);

    let tokenizer = Tokenizer::new(MetadataSink::default(), TokenizerOpts::default());
    let input = BufferQueue::default();

    let mut lines = reader.lines();
    while let Some(Ok(line)) = lines.next() {
        input.push_back(line.into());
        let _ = tokenizer.feed(&input);
    }

    tokenizer.end();

    let mut schema_org = tokenizer.sink.schema_org.into_inner();
    schema_org.sort();

    Metadata {
        basic: tokenizer.sink.basic.into_inner(),
        feeds: tokenizer.sink.feeds.into_inner(),
        open_graph: tokenizer.sink.open_graph.into_inner(),
        schema_org,
    }
}
