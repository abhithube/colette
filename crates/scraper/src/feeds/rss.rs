use colette_core::feed::{ExtractedEntry, ExtractedFeed};

#[derive(Debug, serde::Deserialize)]
pub struct RSSFeed {
    channel: RSSChannel,
}

#[derive(Debug, serde::Deserialize)]
pub struct RSSChannel {
    link: String,
    title: String,
    #[serde(rename = "item")]
    items: Vec<RSSItem>,
}

#[derive(Debug, serde::Deserialize)]
pub struct RSSItem {
    title: String,
    link: String,
    description: String,
    #[serde(rename = "pubDate")]
    pub_date: Option<String>,
    author: Option<String>,
    enclosure: Option<RSSEnclosure>,
    #[serde(rename = "dc:creator")]
    dc_creator: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct RSSEnclosure {
    #[serde(rename = "@url")]
    url: String,
    #[serde(rename = "@type")]
    file_type: String,
}

impl From<RSSFeed> for ExtractedFeed {
    fn from(value: RSSFeed) -> Self {
        Self {
            link: Some(value.channel.link),
            title: Some(value.channel.title),
            entries: value
                .channel
                .items
                .into_iter()
                .map(ExtractedEntry::from)
                .collect(),
        }
    }
}

impl From<RSSItem> for ExtractedEntry {
    fn from(value: RSSItem) -> Self {
        let thumbnail = match value.enclosure {
            Some(enclosure) => {
                if enclosure.file_type.starts_with("image/") {
                    Some(enclosure.url)
                } else {
                    None
                }
            }
            None => None,
        };

        Self {
            link: Some(value.link),
            title: Some(value.title),
            published: value.pub_date,
            description: Some(value.description),
            author: value.dc_creator.or(value.author),
            thumbnail,
        }
    }
}
