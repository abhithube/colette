use rss::{Channel, Item};

use super::{ExtractedFeed, ExtractedFeedEntry};

impl From<Channel> for ExtractedFeed {
    fn from(value: Channel) -> Self {
        Self {
            link: Some(value.link),
            title: Some(value.title),
            entries: value
                .items
                .into_iter()
                .map(ExtractedFeedEntry::from)
                .collect(),
        }
    }
}

impl From<Item> for ExtractedFeedEntry {
    fn from(value: Item) -> Self {
        let mut author = value.author;
        if let Some(mut dc) = value.dublin_core_ext {
            if !dc.creators.is_empty() {
                author = Some(dc.creators.swap_remove(0))
            }
        }

        let thumbnail = value.enclosure.and_then(|e| {
            if e.mime_type.starts_with("image/") {
                Some(e.url)
            } else {
                None
            }
        });

        Self {
            link: value.link,
            title: value.title,
            published: value.pub_date,
            description: value.description,
            author,
            thumbnail,
        }
    }
}
