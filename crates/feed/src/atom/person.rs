use std::io::BufRead;

use quick_xml::{events::Event, Reader};

#[derive(Debug, Clone, Default)]
pub struct AtomPerson {
    pub name: String,
    pub uri: Option<String>,
}

#[derive(Debug, Clone)]
enum PersonTag {
    Name,
    Uri,
}

pub(crate) fn from_reader<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<AtomPerson, anyhow::Error> {
    let mut person = AtomPerson::default();

    let mut tag_stack: Vec<PersonTag> = Vec::new();

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
