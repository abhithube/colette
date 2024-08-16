use colette_core::feed::{ExtractedFeed, ExtractedFeedEntry};

#[derive(Debug, serde::Deserialize)]
pub struct AtomFeed {
    #[serde(rename = "link")]
    links: Vec<AtomLink>,
    title: String,
    #[serde(rename = "entry")]
    entries: Vec<AtomEntry>,
}

#[derive(Debug, serde::Deserialize)]
pub struct AtomEntry {
    #[serde(rename = "link")]
    links: Vec<AtomLink>,
    title: String,
    published: String,
    summary: Option<String>,
    content: Option<String>,
    author: Option<AtomAuthor>,
    group: Option<MediaGroup>,
    thumbnail: Option<MediaThumbnail>,
}

#[derive(Debug, serde::Deserialize)]
pub struct AtomLink {
    #[serde(rename = "@rel")]
    rel: Option<AtomRel>,
    #[serde(rename = "@href")]
    href: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AtomRel {
    #[serde(rename = "self")]
    SelfVal,
    Alternate,
}

#[derive(Debug, serde::Deserialize)]
pub struct AtomAuthor {
    name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct MediaGroup {
    title: String,
    description: String,
    thumbnail: Option<MediaThumbnail>,
}

#[derive(Debug, serde::Deserialize)]
pub struct MediaThumbnail {
    #[serde(rename = "@url")]
    url: String,
}

impl From<AtomFeed> for ExtractedFeed {
    fn from(value: AtomFeed) -> Self {
        Self {
            link: parse_atom_link(value.links),
            title: Some(value.title),
            entries: value
                .entries
                .into_iter()
                .map(ExtractedFeedEntry::from)
                .collect(),
        }
    }
}

impl From<AtomEntry> for ExtractedFeedEntry {
    fn from(value: AtomEntry) -> Self {
        let (title, description, thumbnail) = match value.group {
            Some(group) => (
                group.title,
                Some(group.description),
                value.thumbnail.or(group.thumbnail),
            ),
            None => (
                value.title,
                value.summary.or(value.content),
                value.thumbnail,
            ),
        };

        Self {
            link: parse_atom_link(value.links),
            title: Some(title),
            published: Some(value.published),
            description,
            author: value.author.map(|a| a.name),
            thumbnail: thumbnail.map(|e| e.url),
        }
    }
}

fn parse_atom_link(links: Vec<AtomLink>) -> Option<String> {
    links.into_iter().find_map(|l| match l.rel {
        Some(AtomRel::Alternate) | None => Some(l.href),
        Some(AtomRel::SelfVal) => None,
    })
}
