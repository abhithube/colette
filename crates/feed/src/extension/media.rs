use std::{collections::HashMap, io::BufRead};

use quick_xml::{
    events::{BytesStart, Event},
    Reader,
};

use crate::util::{handle_properties, parse_value, Value};

#[derive(Debug, Clone, Default)]
pub struct MediaGroup {
    pub media_title: Option<String>,
    pub media_description: Option<String>,
    pub media_thumbnail: Vec<MediaThumbnail>,

    pub additional_properties: HashMap<String, Value>,
}

#[derive(Debug, Clone, Default)]
pub struct MediaThumbnail {
    pub url: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
enum MediaGroupTag {
    Title,
    Description,
}

pub(crate) fn from_reader<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<MediaGroup, anyhow::Error> {
    let mut media_group = MediaGroup::default();

    let mut tag_stack: Vec<MediaGroupTag> = Vec::new();

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.name().0).into_owned();

                if tag == "media:title" {
                    tag_stack.push(MediaGroupTag::Title);
                } else if tag == "media:description" {
                    tag_stack.push(MediaGroupTag::Description);
                } else {
                    let value = handle_properties(reader, &e)?;
                    let value = parse_value(reader, buf, tag.clone(), value)?;

                    if let Some(v) = media_group.additional_properties.get_mut(&tag) {
                        match v {
                            Value::Array(arr) => arr.push(value.clone()),
                            _ => *v = Value::Array(vec![v.clone(), value.clone()]),
                        }
                    } else {
                        media_group.additional_properties.insert(tag.clone(), value);
                    }
                }
            }
            Ok(Event::Empty(e)) => {
                let tag = String::from_utf8_lossy(e.name().0).into_owned();

                if tag == "media:thumbnail" {
                    let media_thumbnail = handle_media_thumbnail(reader, &e)?;
                    media_group.media_thumbnail.push(media_thumbnail);
                } else {
                    let value = handle_properties(reader, &e)?;

                    if let Some(v) = media_group.additional_properties.get_mut(&tag) {
                        match v {
                            Value::Array(arr) => arr.push(value.clone()),
                            _ => *v = Value::Array(vec![v.clone(), value.clone()]),
                        }
                    } else {
                        media_group.additional_properties.insert(tag.clone(), value);
                    }
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape()?.into_owned();

                match tag_stack.pop() {
                    Some(MediaGroupTag::Title) => {
                        media_group.media_title = Some(text);
                    }
                    Some(MediaGroupTag::Description) => {
                        media_group.media_description = Some(text);
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                if e.name().0 == b"media:group" {
                    break;
                }
            }
            _ => {}
        }

        buf.clear();
    }

    Ok(media_group)
}

pub(crate) fn handle_media_thumbnail<'a, R: BufRead>(
    reader: &'a Reader<R>,
    e: &'a BytesStart<'a>,
) -> Result<MediaThumbnail, anyhow::Error> {
    let mut media_thumbnail = MediaThumbnail::default();

    for attribute in e.attributes() {
        let attribute = attribute?;

        let value = attribute
            .decode_and_unescape_value(reader.decoder())?
            .into_owned();

        match attribute.key.local_name().into_inner() {
            b"url" => media_thumbnail.url = value,
            b"width" => media_thumbnail.width = value.parse()?,
            b"height" => media_thumbnail.height = value.parse()?,
            _ => {}
        }
    }

    Ok(media_thumbnail)
}
