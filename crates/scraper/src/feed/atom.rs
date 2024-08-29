use atom_syndication::{Entry, Feed, Link};

use super::{extractor::ExtractedFeedEntry, ExtractedFeed};

impl From<Feed> for ExtractedFeed {
    fn from(value: Feed) -> Self {
        Self {
            link: parse_atom_link(value.links),
            title: Some(value.title.value),
            entries: value
                .entries
                .into_iter()
                .map(ExtractedFeedEntry::from)
                .collect(),
        }
    }
}

impl From<Entry> for ExtractedFeedEntry {
    fn from(mut value: Entry) -> Self {
        let mut title = value.title.value;
        let mut description = value
            .summary
            .map(|e| e.value)
            .or(value.content.and_then(|e| e.value));
        let mut thumbnail = Option::<String>::None;

        if let Some(mut media) = value.extensions.remove("media") {
            if let Some(mut group_exts) = media.remove("group") {
                if !group_exts.is_empty() {
                    let mut group = group_exts.swap_remove(0).children;
                    if let Some(mut title_exts) = group.remove("title") {
                        if !title_exts.is_empty() {
                            if let Some(t) = title_exts.swap_remove(0).value {
                                title = t;
                            }
                        }
                    }
                    if let Some(mut description_exts) = group.remove("description") {
                        if !description_exts.is_empty() {
                            description = description_exts.swap_remove(0).value;
                        }
                    }
                    if let Some(mut thumbnail_exts) = group.remove("thumbnail") {
                        if !thumbnail_exts.is_empty() {
                            thumbnail = thumbnail_exts.swap_remove(0).attrs.remove("url");
                        }
                    }
                }
            }
        }

        Self {
            link: parse_atom_link(value.links),
            title: Some(title),
            published: value.published.map(|e| e.to_rfc3339()),
            description,
            author: if !value.authors.is_empty() {
                Some(value.authors.swap_remove(0).name)
            } else {
                None
            },
            thumbnail,
        }
    }
}

fn parse_atom_link(links: Vec<Link>) -> Option<String> {
    links.into_iter().find_map(|l| match l.rel.as_str() {
        "alternate" => Some(l.href),
        _ => None,
    })
}
