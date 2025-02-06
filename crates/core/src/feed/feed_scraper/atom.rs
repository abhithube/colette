use colette_feed::atom::{AtomEntry, AtomFeed, AtomLink, AtomRel};

use super::{ExtractedFeed, ExtractedFeedEntry};

impl From<AtomFeed> for ExtractedFeed {
    fn from(value: AtomFeed) -> Self {
        Self {
            link: parse_atom_link(value.link),
            title: Some(value.title.text),
            entries: value
                .entry
                .into_iter()
                .map(ExtractedFeedEntry::from)
                .collect(),
        }
    }
}

impl From<AtomEntry> for ExtractedFeedEntry {
    fn from(value: AtomEntry) -> Self {
        let mut title = value.title.text;
        let mut description = value.summary.or(value.content).map(|e| e.text);
        let mut thumbnail = Option::<String>::None;

        if let Some(extension) = value.extension {
            if let Some(mut media_group) = extension.media_group {
                if let Some(media_title) = media_group.media_title {
                    title = media_title;
                }
                if media_group.media_description.is_some() {
                    description = media_group.media_description;
                }

                if !media_group.media_thumbnail.is_empty() {
                    let media_thumbnail = media_group.media_thumbnail.swap_remove(0);
                    thumbnail = Some(media_thumbnail.url);
                }
            }
        }

        Self {
            link: parse_atom_link(value.link),
            title: Some(title),
            published: value.published,
            description,
            author: Some(
                value
                    .author
                    .into_iter()
                    .map(|e| e.name)
                    .collect::<Vec<_>>()
                    .join(","),
            ),
            thumbnail,
        }
    }
}

fn parse_atom_link(links: Vec<AtomLink>) -> Option<String> {
    links.into_iter().find_map(|l| match l.rel {
        AtomRel::Alternate => Some(l.href),
        _ => None,
    })
}
