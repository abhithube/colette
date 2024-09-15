use std::io::BufRead;

use anyhow::anyhow;
use quick_xml::{events::Event, Reader};

use crate::{Opml, Outline, OutlineType, Version};

enum OpmlField {
    Title,
}

pub fn from_reader<R: BufRead>(reader: R) -> Result<Opml, anyhow::Error> {
    let mut opml = Opml::default();

    let mut current: Option<OpmlField> = None;
    let mut stack: Vec<Outline> = Vec::new();

    let mut buf: Vec<u8> = Vec::new();
    let mut reader = Reader::from_reader(reader);

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"opml" => {
                    for attribute in e.attributes() {
                        let attribute = attribute?;

                        let value = attribute.decode_and_unescape_value(reader.decoder())?;

                        if attribute.key.local_name().into_inner() == b"version" {
                            opml.version = match value.as_ref() {
                                "1.0" => Ok(Version::V1),
                                "1.1" => Ok(Version::V1_1),
                                "2.0" => Ok(Version::V2),
                                _ => Err(anyhow!("OPML version not supported")),
                            }?;
                        }
                    }
                }
                b"title" => current = Some(OpmlField::Title),
                b"outline" => {
                    let mut outline = Outline::default();

                    for attribute in e.attributes() {
                        let attribute = attribute?;

                        let value = attribute.decode_and_unescape_value(reader.decoder())?;

                        match attribute.key.local_name().into_inner() {
                            b"type" => {
                                if value.as_ref() == "rss" {
                                    outline.r#type = Some(OutlineType::Rss)
                                }
                            }
                            b"text" => outline.text = value.into_owned(),
                            b"xmlUrl" => outline.xml_url = Some(value.into_owned()),
                            b"title" => outline.title = Some(value.into_owned()),
                            b"htmlUrl" => outline.html_url = Some(value.into_owned()),
                            _ => {}
                        }
                    }

                    stack.push(outline);
                }
                _ => {}
            },
            Ok(Event::Text(e)) => {
                if let Some(OpmlField::Title) = current {
                    opml.head.title = e.unescape()?.into_owned();

                    current = None;
                }
            }
            Ok(Event::End(e)) => {
                if e.name().as_ref() == b"outline" {
                    if let Some(outline) = stack.pop() {
                        if let Some(parent) = stack.last_mut() {
                            parent.outline.get_or_insert_with(Vec::new).push(outline);
                        } else {
                            opml.body.outlines.push(outline);
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            _ => {}
        }

        buf.clear();
    }

    Ok(opml)
}
