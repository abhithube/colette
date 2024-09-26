use std::io::BufRead;

use anyhow::anyhow;
use quick_xml::{events::Event, Reader};

#[derive(Debug, Clone)]
pub enum Feed {
    Atom,
    Rss,
}

pub fn from_reader<R: BufRead>(reader: R) -> Result<Feed, anyhow::Error> {
    let mut reader = Reader::from_reader(reader);
    reader.config_mut().trim_text(true);

    let mut buf: Vec<u8> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.name().0 {
                b"feed" => return Ok(Feed::Atom),
                b"rss" => return Ok(Feed::Rss),
                _ => {}
            },
            Ok(Event::Eof) => break,
            _ => (),
        }

        buf.clear();
    }

    Err(anyhow!("feed not supported"))
}
