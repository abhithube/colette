use colette_feed::rss::{RssFeed, RssItem};

use super::{ExtractedFeed, ExtractedFeedEntry};

impl From<RssFeed> for ExtractedFeed {
    fn from(value: RssFeed) -> Self {
        Self {
            link: Some(value.channel.link),
            title: Some(value.channel.title),
            entries: value
                .channel
                .item
                .into_iter()
                .map(ExtractedFeedEntry::from)
                .collect(),
        }
    }
}

impl From<RssItem> for ExtractedFeedEntry {
    fn from(value: RssItem) -> Self {
        Self {
            link: Some(value.link),
            title: Some(value.title),
            published: value.pub_date,
            description: Some(value.description),
            author: value.author,
            thumbnail: None,
        }
    }
}
