use std::{collections::HashMap, io::BufRead};

use quick_xml::{Reader, events::Event};

use crate::{
    Error,
    rss::item::{self, RssItem},
    util::{Value, handle_properties, parse_value},
};

#[derive(Debug, Clone, Default)]
pub struct RssChannel {
    pub link: String,
    pub title: String,
    pub description: String,
    pub pub_date: Option<String>,
    pub item: Vec<RssItem>,

    pub additional_properties: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
enum ChannelTag {
    Title,
    Link,
    Description,
    PubDate,
}

pub(crate) fn from_reader<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<RssChannel, Error> {
    let mut channel = RssChannel::default();

    let mut tag_stack: Vec<ChannelTag> = Vec::new();

    fn handle_text(tag_stack: &mut Vec<ChannelTag>, channel: &mut RssChannel, text: String) {
        match tag_stack.pop() {
            Some(ChannelTag::Title) => {
                channel.title = text;
            }
            Some(ChannelTag::Link) => {
                channel.link = text;
            }
            Some(ChannelTag::Description) => {
                channel.description = text;
            }
            Some(ChannelTag::PubDate) => {
                channel.pub_date = Some(text);
            }
            _ => {}
        }
    }

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.name().0).into_owned();

                if tag == "title" {
                    tag_stack.push(ChannelTag::Title);
                } else if tag == "link" {
                    tag_stack.push(ChannelTag::Link);
                } else if tag == "description" {
                    tag_stack.push(ChannelTag::Description);
                } else if tag == "pubDate" {
                    tag_stack.push(ChannelTag::PubDate);
                } else if tag == "item" {
                    let item = item::from_reader(reader, buf)?;
                    channel.item.push(item);
                } else {
                    let value = handle_properties(reader, &e)?;
                    let value = parse_value(reader, buf, tag.clone(), value)?;

                    if let Some(v) = channel.additional_properties.get_mut(&tag) {
                        match v {
                            Value::Array(arr) => arr.push(value.clone()),
                            _ => *v = Value::Array(vec![v.clone(), value.clone()]),
                        }
                    } else {
                        channel.additional_properties.insert(tag.clone(), value);
                    }
                }
            }
            Ok(Event::Empty(e)) => {
                let tag = String::from_utf8_lossy(e.name().0).into_owned();

                let value = handle_properties(reader, &e)?;

                if let Some(v) = channel.additional_properties.get_mut(&tag) {
                    match v {
                        Value::Array(arr) => arr.push(value.clone()),
                        _ => *v = Value::Array(vec![v.clone(), value.clone()]),
                    }
                } else {
                    channel.additional_properties.insert(tag.clone(), value);
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape()?.into_owned();

                handle_text(&mut tag_stack, &mut channel, text);
            }
            Ok(Event::CData(e)) => {
                let text = String::from_utf8_lossy(&e.into_inner()).into_owned();

                handle_text(&mut tag_stack, &mut channel, text);
            }
            Ok(Event::End(e)) => {
                if e.name().0 == b"channel" {
                    break;
                }
            }
            _ => (),
        }

        buf.clear();
    }

    Ok(channel)
}
