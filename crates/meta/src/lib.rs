use std::{
    cell::{Cell, RefCell},
    io::{BufRead, BufReader, Read},
};

use basic::{handle_basic, Basic};
use html5ever::{
    tendril::StrTendril,
    tokenizer::{
        BufferQueue, CharacterTokens, EndTag, StartTag, Tag, TagToken, Token, TokenSink,
        TokenSinkResult, Tokenizer, TokenizerOpts,
    },
};
use open_graph::{handle_open_graph, OpenGraph};
use rss::{handle_rss, Feed};
use schema_org::{
    handle_json_ld, handle_microdata, Article, ImageObject, Person, SchemaObject,
    SchemaObjectOrValue, VideoObject, WebPage, WebSite,
};
use serde_json::Value;

pub mod basic;
pub mod open_graph;
pub mod rss;
pub mod schema_org;

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
            CharacterTokens(inner_text) => self.handle_inner_text(inner_text),
            TagToken(tag) => match tag.kind {
                StartTag => self.handle_start_tag(tag),
                EndTag => self.handle_end_tag(tag),
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
            (_, _, Some(itemtype), _, _, _, _) => match itemtype.split("/").last() {
                Some("Article") => {
                    let mut stack = self.schema_stack.borrow_mut();
                    stack.push(SchemaObjectOrValue::SchemaObject(SchemaObject::Article(
                        Article::default(),
                    )));
                }
                Some("ImageObject") => {
                    let mut stack = self.schema_stack.borrow_mut();
                    stack.push(SchemaObjectOrValue::SchemaObject(
                        SchemaObject::ImageObject(ImageObject::default()),
                    ));
                }
                Some("Person") => {
                    let mut stack = self.schema_stack.borrow_mut();
                    stack.push(SchemaObjectOrValue::SchemaObject(SchemaObject::Person(
                        Person::default(),
                    )));
                }
                Some("VideoObject") => {
                    let mut stack = self.schema_stack.borrow_mut();
                    stack.push(SchemaObjectOrValue::SchemaObject(
                        SchemaObject::VideoObject(VideoObject::default()),
                    ));
                }
                Some("WebPage") => {
                    let mut stack = self.schema_stack.borrow_mut();
                    stack.push(SchemaObjectOrValue::SchemaObject(SchemaObject::WebPage(
                        WebPage::default(),
                    )));
                }
                Some("WebSite") => {
                    let mut stack = self.schema_stack.borrow_mut();
                    stack.push(SchemaObjectOrValue::SchemaObject(SchemaObject::WebSite(
                        WebSite::default(),
                    )));
                }
                _ => {}
            },
            (content, Some(itemprop), _, href, _, _, _) => {
                if let Some(current) = self.schema_stack.borrow_mut().last_mut() {
                    self.current_itemprop.replace(Some(itemprop.clone()));

                    let content = content.unwrap_or(href.unwrap_or_default());
                    handle_microdata(current, itemprop.into(), content.into());
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
        }

        let mut stack = self.schema_stack.borrow_mut();
        if let (Some(itemprop), Some(completed_schema)) =
            (self.current_itemprop.take(), stack.pop())
        {
            if let Some(parent_schema) = stack.last_mut() {
                let mut author: Option<Person> = None;
                let mut image: Option<ImageObject> = None;
                let mut thumbnail: Option<ImageObject> = None;
                let mut value: Option<Value> = None;

                match completed_schema {
                    SchemaObjectOrValue::SchemaObject(schema) => match schema {
                        SchemaObject::ImageObject(image_object) => match itemprop.as_ref() {
                            "image" => image = Some(image_object),
                            "thumbnail" => thumbnail = Some(image_object),
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
                            article.author = author;
                            article.image = image;
                            article.thumbnail = thumbnail;
                        }
                        SchemaObject::ImageObject(image_object) => {
                            image_object.author = author.map(Box::new);
                            image_object.image = image.map(Box::new);
                            image_object.thumbnail = thumbnail.map(Box::new);
                        }
                        SchemaObject::Person(person) => {
                            person.image = image;
                        }
                        SchemaObject::VideoObject(video_object) => {
                            video_object.author = author;
                            video_object.image = image;
                            video_object.thumbnail = thumbnail;
                        }
                        SchemaObject::WebPage(webpage) => {
                            webpage.author = author;
                            webpage.image = image;
                            webpage.thumbnail = thumbnail;
                        }
                        SchemaObject::WebSite(website) => {
                            website.author = author;
                            website.image = image;
                            website.thumbnail = thumbnail;
                        }
                    },
                    SchemaObjectOrValue::Other(other) => {
                        if let Value::Object(object) = other {
                            if let (Some(value), Some(itemprop)) =
                                (value, self.current_itemprop.take())
                            {
                                object.insert(itemprop.into(), value);
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

    Metadata {
        basic: tokenizer.sink.basic.into_inner(),
        feeds: tokenizer.sink.feeds.into_inner(),
        open_graph: tokenizer.sink.open_graph.into_inner(),
        schema_org: tokenizer.sink.schema_org.into_inner(),
    }
}
