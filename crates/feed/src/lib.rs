use std::io::BufRead;

use anyhow::anyhow;
use atom::AtomFeed;
use quick_xml::{events::Event, Reader};
use rss::RssFeed;

pub mod atom;
pub mod extension;
pub mod rss;
pub mod util;

#[derive(Debug, Clone)]
pub enum Feed {
    Atom(AtomFeed),
    Rss(RssFeed),
}

pub fn from_reader<R: BufRead>(reader: R) -> Result<Feed, anyhow::Error> {
    let mut reader = Reader::from_reader(reader);
    reader.config_mut().trim_text(true);

    let mut buf: Vec<u8> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.name().0 {
                b"feed" => {
                    let atom = atom::from_reader(&mut reader, &mut buf)?;
                    return Ok(Feed::Atom(atom));
                }
                b"rss" => {
                    let rss = rss::from_reader(&mut reader, &mut buf)?;
                    return Ok(Feed::Rss(rss));
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            _ => (),
        }

        buf.clear();
    }

    Err(anyhow!("feed not supported"))
}
