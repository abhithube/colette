use std::{collections::HashMap, io::BufRead};

use quick_xml::{events::Event, Reader};

use super::{handle_link, handle_text, person, AtomLink, AtomPerson, AtomText};
use crate::{
    extension::{
        media::{self, handle_media_thumbnail, MediaGroup},
        Extension,
    },
    util::{handle_properties, parse_value, Value},
};

#[derive(Debug, Clone, Default)]
pub struct AtomEntry {
    pub link: Vec<AtomLink>,
    pub title: AtomText,
    pub published: Option<String>,
    pub author: Vec<AtomPerson>,
    pub summary: Option<AtomText>,
    pub content: Option<AtomText>,

    pub extension: Option<Extension>,

    pub additional_properties: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
enum EntryTag {
    Title(AtomText),
    Published,
    Summary(AtomText),
    Content(AtomText),
}

pub(crate) fn from_reader<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<AtomEntry, anyhow::Error> {
    let mut entry = AtomEntry::default();

    let mut tag_stack: Vec<EntryTag> = Vec::new();

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
                    let author = person::from_reader(reader, buf)?;
                    entry.author.push(author);
                } else if tag == "summary" {
                    let text = handle_text(reader, &e)?;
                    tag_stack.push(EntryTag::Summary(text));
                } else if tag == "content" {
                    let text = handle_text(reader, &e)?;
                    tag_stack.push(EntryTag::Content(text));
                } else if tag == "media:group" {
                    let media_group = media::from_reader(reader, buf)?;
                    entry
                        .extension
                        .get_or_insert_with(Extension::default)
                        .media_group = Some(media_group);
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
                } else if tag == "media:thumbnail" {
                    let media_thumbnail = handle_media_thumbnail(reader, &e)?;
                    entry
                        .extension
                        .get_or_insert_with(Extension::default)
                        .media_group
                        .get_or_insert_with(MediaGroup::default)
                        .media_thumbnail
                        .push(media_thumbnail);
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
