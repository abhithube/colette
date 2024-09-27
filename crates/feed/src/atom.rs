use core::str;
use std::{collections::HashMap, io::BufRead, str::FromStr};

use quick_xml::{
    events::{BytesStart, Event},
    Reader,
};

use crate::util::{handle_properties, parse_value, Value};

#[derive(Debug, Clone, Default)]
pub struct AtomFeed {
    pub link: Vec<AtomLink>,
    pub title: AtomText,
    pub published: Option<String>,
    pub author: Vec<AtomPerson>,
    pub entry: Vec<AtomEntry>,

    pub additional_properties: HashMap<String, Value>,
}

#[derive(Debug, Clone, Default)]
pub struct AtomEntry {
    pub link: Vec<AtomLink>,
    pub title: AtomText,
    pub published: Option<String>,
    pub author: Vec<AtomPerson>,
    pub summary: Option<AtomText>,
    pub content: Option<AtomText>,

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
    type Err = anyhow::Error;

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
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "html" => Ok(AtomTextType::Html),
            "xhtml" => Ok(AtomTextType::Xhtml),
            _ => Ok(AtomTextType::default()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AtomPerson {
    pub name: String,
    pub uri: Option<String>,
}

#[derive(Debug, Clone)]
enum FeedTag {
    Title(AtomText),
    Published,
}

pub(crate) fn parse_atom<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<AtomFeed, anyhow::Error> {
    let mut feed = AtomFeed::default();

    let mut tag_stack: Vec<FeedTag> = Vec::new();

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.name().0).into_owned();

                if tag == "title" {
                    let text = handle_text(reader, &e)?;
                    tag_stack.push(FeedTag::Title(text));
                } else if tag == "published" {
                    tag_stack.push(FeedTag::Published);
                } else if tag == "author" {
                    let author = parse_person(reader, buf)?;
                    feed.author.push(author);
                } else if tag == "entry" {
                    let entry = parse_entry(reader, buf)?;
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
                    Some(FeedTag::Published) => {
                        feed.published = Some(text);
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

#[derive(Debug, Clone)]
enum EntryTag {
    Title(AtomText),
    Published,
    Summary(AtomText),
    Content(AtomText),
}

fn parse_entry<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<AtomEntry, anyhow::Error> {
    let mut entry = AtomEntry::default();

    let mut tag_stack: Vec<EntryTag> = vec![];

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.name().0).into_owned();

                if tag == "title" {
                    let text = handle_text(reader, &e)?;
                    tag_stack.push(EntryTag::Title(text));
                } else if tag == "published" {
                    tag_stack.push(EntryTag::Published);
                } else if tag == "author" {
                    let author = parse_person(reader, buf)?;
                    entry.author.push(author);
                } else if tag == "summary" {
                    let text = handle_text(reader, &e)?;
                    tag_stack.push(EntryTag::Summary(text));
                } else if tag == "content" {
                    let text = handle_text(reader, &e)?;
                    tag_stack.push(EntryTag::Content(text));
                } else {
                    let value = handle_properties(reader, &e)?;
                    let value = parse_value(reader, buf, tag.clone(), value)?;

                    if let Some(v) = entry.additional_properties.get_mut(&tag) {
                        match v {
                            Value::Array(arr) => arr.push(value.clone()),
                            _ => *v = Value::Array(vec![v.clone(), value.clone()]),
                        }
                    } else {
                        entry.additional_properties.insert(tag.clone(), value);
                    }
                }
            }
            Ok(Event::Empty(e)) => {
                let tag = String::from_utf8_lossy(e.name().0).into_owned();

                if tag == "link" {
                    let link = handle_link(reader, &e)?;
                    entry.link.push(link);
                } else {
                    let value = handle_properties(reader, &e)?;

                    if let Some(v) = entry.additional_properties.get_mut(&tag) {
                        match v {
                            Value::Array(arr) => arr.push(value.clone()),
                            _ => *v = Value::Array(vec![v.clone(), value.clone()]),
                        }
                    } else {
                        entry.additional_properties.insert(tag.clone(), value);
                    }
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape()?.into_owned();

                match tag_stack.pop() {
                    Some(EntryTag::Title(mut t)) => {
                        t.text = text;
                        entry.title = t;
                    }
                    Some(EntryTag::Published) => {
                        entry.published = Some(text);
                    }
                    Some(EntryTag::Summary(mut t)) => {
                        t.text = text;
                        entry.summary = Some(t);
                    }
                    Some(EntryTag::Content(mut t)) => {
                        t.text = text;
                        entry.content = Some(t);
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                if e.name().0 == b"entry" {
                    break;
                }
            }
            _ => {}
        }

        buf.clear();
    }

    Ok(entry)
}

#[derive(Debug, Clone)]
enum PersonTag {
    Name,
    Uri,
}

fn parse_person<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<AtomPerson, anyhow::Error> {
    let mut person = AtomPerson::default();

    let mut tag_stack: Vec<PersonTag> = vec![];

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => match e.name().0 {
                b"name" => tag_stack.push(PersonTag::Name),
                b"uri" => tag_stack.push(PersonTag::Uri),
                _ => {}
            },
            Ok(Event::Text(e)) => {
                let text = e.unescape()?.into_owned();

                match tag_stack.pop() {
                    Some(PersonTag::Name) => {
                        person.name = text;
                    }
                    Some(PersonTag::Uri) => {
                        person.uri = Some(text);
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                if e.name().0 == b"author" {
                    break;
                }
            }
            _ => {}
        }

        buf.clear();
    }

    Ok(person)
}

pub(crate) fn handle_link<'a, R: BufRead>(
    reader: &'a Reader<R>,
    e: &'a BytesStart<'a>,
) -> Result<AtomLink, anyhow::Error> {
    let mut link = AtomLink::default();

    for attribute in e.attributes() {
        let attribute = attribute?;

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
) -> Result<AtomText, anyhow::Error> {
    let mut text = AtomText::default();

    for attribute in e.attributes() {
        let attribute = attribute?;

        let value = attribute
            .decode_and_unescape_value(reader.decoder())?
            .into_owned();

        if attribute.key.local_name().into_inner() == b"type" {
            text.r#type = value.parse()?
        }
    }

    Ok(text)
}
