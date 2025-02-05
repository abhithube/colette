use std::{collections::HashMap, io::BufRead};

use quick_xml::{events::Event, Reader};

use crate::{
    util::{handle_properties, parse_value, Value},
    Error,
};

#[derive(Debug, Clone, Default)]
pub struct RssItem {
    pub link: String,
    pub title: String,
    pub description: String,
    pub pub_date: Option<String>,
    pub author: Option<String>,

    pub additional_properties: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
enum ItemTag {
    Title,
    Link,
    Description,
    PubDate,
    Author,
}

pub(crate) fn from_reader<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<RssItem, Error> {
    let mut item = RssItem::default();

    let mut tag_stack: Vec<ItemTag> = Vec::new();

    fn handle_text(tag_stack: &mut Vec<ItemTag>, item: &mut RssItem, text: String) {
        match tag_stack.pop() {
            Some(ItemTag::Title) => {
                item.title = text;
            }
            Some(ItemTag::Link) => {
                item.link = text;
            }
            Some(ItemTag::Description) => {
                item.description = text;
            }
            Some(ItemTag::PubDate) => {
                item.pub_date = Some(text);
            }
            Some(ItemTag::Author) => {
                item.author = Some(text);
            }
            _ => {}
        }
    }

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.name().0).into_owned();

                if tag == "title" {
                    tag_stack.push(ItemTag::Title);
                } else if tag == "link" {
                    tag_stack.push(ItemTag::Link);
                } else if tag == "description" {
                    tag_stack.push(ItemTag::Description);
                } else if tag == "pubDate" {
                    tag_stack.push(ItemTag::PubDate);
                } else if tag == "author" {
                    tag_stack.push(ItemTag::Author);
                } else {
                    let value = handle_properties(reader, &e)?;
                    let value = parse_value(reader, buf, tag.clone(), value)?;

                    if let Some(v) = item.additional_properties.get_mut(&tag) {
                        match v {
                            Value::Array(arr) => arr.push(value.clone()),
                            _ => *v = Value::Array(vec![v.clone(), value.clone()]),
                        }
                    } else {
                        item.additional_properties.insert(tag.clone(), value);
                    }
                }
            }
            Ok(Event::Empty(e)) => {
                let tag = String::from_utf8_lossy(e.name().0).into_owned();

                let value = handle_properties(reader, &e)?;

                if let Some(v) = item.additional_properties.get_mut(&tag) {
                    match v {
                        Value::Array(arr) => arr.push(value.clone()),
                        _ => *v = Value::Array(vec![v.clone(), value.clone()]),
                    }
                } else {
                    item.additional_properties.insert(tag.clone(), value);
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape()?.into_owned();

                handle_text(&mut tag_stack, &mut item, text);
            }
            Ok(Event::CData(e)) => {
                let text = String::from_utf8_lossy(&e.into_inner()).into_owned();

                handle_text(&mut tag_stack, &mut item, text);
            }
            Ok(Event::End(e)) => {
                if e.name().0 == b"item" {
                    break;
                }
            }
            _ => (),
        }

        buf.clear();
    }

    Ok(item)
}
