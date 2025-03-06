use core::str;
use std::{collections::HashMap, io::BufRead, str::FromStr};

pub use entry::AtomEntry;
pub use person::AtomPerson;
use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};

use crate::{
    Error,
    util::{Value, handle_properties, parse_value},
};

mod entry;
mod person;

#[derive(Debug, Clone, Default)]
pub struct AtomFeed {
    pub link: Vec<AtomLink>,
    pub title: AtomText,
    pub subtitle: Option<AtomText>,
    pub updated: String,
    pub author: Vec<AtomPerson>,
    pub entry: Vec<AtomEntry>,

    pub additional_properties: HashMap<String, Value>,
}

#[derive(Debug, Clone, Default)]
pub struct AtomLink {
    pub rel: AtomRel,
    pub r#type: Option<String>,
    pub href: String,
}

#[derive(Debug, Clone, Default)]
pub enum AtomRel {
    #[default]
    Alternate,
    RelSelf,
}

impl FromStr for AtomRel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "alternative" => Ok(AtomRel::Alternate),
            "self" => Ok(AtomRel::RelSelf),
            _ => Ok(AtomRel::default()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AtomText {
    pub r#type: AtomTextType,
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub enum AtomTextType {
    #[default]
    Text,
    Html,
    Xhtml,
}

impl FromStr for AtomTextType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "html" => Ok(AtomTextType::Html),
            "xhtml" => Ok(AtomTextType::Xhtml),
            _ => Ok(AtomTextType::default()),
        }
    }
}

#[derive(Debug, Clone)]
enum FeedTag {
    Title(AtomText),
    Subtitle(AtomText),
    Updated,
}

pub fn from_reader<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<AtomFeed, Error> {
    let mut feed = AtomFeed::default();

    let mut tag_stack: Vec<FeedTag> = Vec::new();

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.name().0).into_owned();

                if tag == "title" {
                    let text = handle_text(reader, &e)?;
                    tag_stack.push(FeedTag::Title(text));
                } else if tag == "subtitle" {
                    let text = handle_text(reader, &e)?;
                    tag_stack.push(FeedTag::Subtitle(text));
                } else if tag == "updated" {
                    tag_stack.push(FeedTag::Updated);
                } else if tag == "author" {
                    let author = person::from_reader(reader, buf)?;
                    feed.author.push(author);
                } else if tag == "entry" {
                    let entry = entry::from_reader(reader, buf)?;
                    feed.entry.push(entry);
                } else {
                    let value = handle_properties(reader, &e)?;
                    let value = parse_value(reader, buf, tag.clone(), value)?;

                    if let Some(v) = feed.additional_properties.get_mut(&tag) {
                        match v {
                            Value::Array(arr) => arr.push(value.clone()),
                            _ => *v = Value::Array(vec![v.clone(), value.clone()]),
                        }
                    } else {
                        feed.additional_properties.insert(tag.clone(), value);
                    }
                }
            }
            Ok(Event::Empty(e)) => {
                let tag = String::from_utf8_lossy(e.name().0).into_owned();

                if tag == "link" {
                    let link = handle_link(reader, &e)?;
                    feed.link.push(link);
                } else {
                    let value = handle_properties(reader, &e)?;

                    if let Some(v) = feed.additional_properties.get_mut(&tag) {
                        match v {
                            Value::Array(arr) => arr.push(value.clone()),
                            _ => *v = Value::Array(vec![v.clone(), value.clone()]),
                        }
                    } else {
                        feed.additional_properties.insert(tag.clone(), value);
                    }
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape()?.into_owned();

                match tag_stack.pop() {
                    Some(FeedTag::Title(mut t)) => {
                        t.text = text;
                        feed.title = t;
                    }
                    Some(FeedTag::Subtitle(mut t)) => {
                        t.text = text;
                        feed.subtitle = Some(t);
                    }
                    Some(FeedTag::Updated) => {
                        feed.updated = text;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                if e.name().0 == b"feed" {
                    break;
                }
            }
            _ => (),
        }

        buf.clear();
    }

    Ok(feed)
}

fn handle_link<'a, R: BufRead>(
    reader: &'a Reader<R>,
    e: &'a BytesStart<'a>,
) -> Result<AtomLink, Error> {
    let mut link = AtomLink::default();

    for attribute in e.attributes() {
        let attribute = attribute.map_err(|e| Error::Parse(e.into()))?;

        let value = attribute
            .decode_and_unescape_value(reader.decoder())?
            .into_owned();

        match attribute.key.local_name().into_inner() {
            b"rel" => link.rel = value.parse()?,
            b"type" => link.r#type = Some(value),
            b"href" => link.href = value,
            _ => {}
        }
    }

    Ok(link)
}

fn handle_text<'a, R: BufRead>(
    reader: &'a Reader<R>,
    e: &'a BytesStart<'a>,
) -> Result<AtomText, Error> {
    let mut text = AtomText::default();

    for attribute in e.attributes() {
        let attribute = attribute.map_err(|e| Error::Parse(e.into()))?;

        let value = attribute
            .decode_and_unescape_value(reader.decoder())?
            .into_owned();

        if attribute.key.local_name().into_inner() == b"type" {
            text.r#type = value.parse()?
        }
    }

    Ok(text)
}
