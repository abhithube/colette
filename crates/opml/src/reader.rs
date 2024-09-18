use std::io::BufRead;

use quick_xml::{
    events::{BytesStart, Event},
    Reader,
};

use crate::{Opml, Outline};

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
                            opml.version = value.parse()?;
                        }
                    }
                }
                b"title" => current = Some(OpmlField::Title),
                b"outline" => {
                    let outline = handle_outline(&reader, e)?;

                    stack.push(outline);
                }
                _ => {}
            },
            Ok(Event::Empty(e)) => {
                if e.name().as_ref() == b"outline" {
                    let outline = handle_outline(&reader, e)?;

                    if let Some(parent) = stack.last_mut() {
                        parent.outline.get_or_insert_with(Vec::new).push(outline);
                    } else {
                        opml.body.outlines.push(outline);
                    }
                }
            }
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

fn handle_outline<R: BufRead>(
    reader: &Reader<R>,
    tag: BytesStart,
) -> Result<Outline, anyhow::Error> {
    let mut outline = Outline::default();

    for attribute in tag.attributes() {
        let attribute = attribute?;

        let value = attribute.decode_and_unescape_value(reader.decoder())?;
        match attribute.key.local_name().into_inner() {
            b"type" => outline.r#type = Some(value.parse()?),
            b"text" => outline.text = value.into_owned(),
            b"xmlUrl" => outline.xml_url = Some(value.into_owned()),
            b"title" => outline.title = Some(value.into_owned()),
            b"htmlUrl" => outline.html_url = Some(value.into_owned()),
            _ => {}
        }
    }

    Ok(outline)
}
