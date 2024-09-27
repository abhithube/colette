use std::{collections::HashMap, io::BufRead};

use quick_xml::{events::Event, Reader};

use crate::util::{handle_properties, parse_value, Value};

#[derive(Debug, Clone, Default)]
pub struct RssFeed {
    pub additional_properties: HashMap<String, Value>,
}

pub(crate) fn parse_rss<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<RssFeed, anyhow::Error> {
    let mut feed = RssFeed::default();

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.name().0).into_owned();

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
            Ok(Event::Empty(e)) => {
                let tag = String::from_utf8_lossy(e.name().0).into_owned();

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
            Ok(Event::End(e)) => {
                if e.name().0 == b"rss" {
                    break;
                }
            }
            _ => (),
        }

        buf.clear();
    }

    Ok(feed)
}
