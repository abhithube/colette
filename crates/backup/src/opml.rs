use colette_core::{
    backup::{self, BackupManager},
    feeds::BackupFeed,
};
use quick_xml::se::Serializer;
use serde::Serialize;
use url::Url;

#[derive(Default)]
pub struct OpmlManager;

impl BackupManager for OpmlManager {
    type T = Vec<BackupFeed>;

    fn import(&self, raw: &str) -> Result<Self::T, backup::Error> {
        let result =
            quick_xml::de::from_str::<Opml>(raw).map_err(|_| backup::Error::Deserialize)?;

        let mut feeds = Vec::<BackupFeed>::new();
        extract_feeds(result.body.outlines, &mut feeds);

        Ok(feeds)
    }

    fn export(&self, data: Self::T) -> Result<String, backup::Error> {
        let opml = Opml {
            version: String::from("2.0"),
            head: Head {
                title: String::from("Feeds"),
            },
            body: Body {
                outlines: data.into_iter().map(Outline::from).collect(),
            },
        };

        let mut buffer = String::new();
        let mut ser = Serializer::with_root(&mut buffer, Some("opml"))
            .map_err(|_| backup::Error::Serialize)?;
        ser.indent(' ', 2);

        opml.serialize(ser)
            .map_err(|_| backup::Error::Deserialize)?;

        let raw = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n".to_owned() + &buffer;

        Ok(raw)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Opml {
    #[serde(rename = "@version")]
    version: String,
    head: Head,
    body: Body,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Head {
    title: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Body {
    #[serde(rename = "outline")]
    outlines: Vec<Outline>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Outline {
    #[serde(rename = "@text")]
    text: String,
    #[serde(rename = "@type")]
    outline_type: Option<OutlineType>,
    #[serde(rename = "@xmlUrl")]
    xml_url: Option<Url>,
    #[serde(rename = "@title")]
    title: Option<String>,
    #[serde(rename = "@htmlUrl")]
    html_url: Option<Url>,
    #[serde(rename = "outline")]
    children: Option<Vec<Outline>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum OutlineType {
    Rss,
    Atom,
}

impl From<Outline> for BackupFeed {
    fn from(value: Outline) -> Self {
        Self {
            title: value.title.unwrap_or(value.text),
            xml_url: value.xml_url.unwrap(),
            html_url: value.html_url,
        }
    }
}

impl From<BackupFeed> for Outline {
    fn from(value: BackupFeed) -> Self {
        Self {
            text: value.title.clone(),
            title: Some(value.title),
            outline_type: Some(OutlineType::Rss),
            xml_url: Some(value.xml_url),
            html_url: value.html_url,
            children: None,
        }
    }
}

fn extract_feeds(outlines: Vec<Outline>, feeds: &mut Vec<BackupFeed>) {
    for outline in outlines {
        if outline.outline_type.is_some() {
            feeds.push(BackupFeed::from(outline));
        } else if let Some(children) = outline.children {
            extract_feeds(children, feeds);
        }
    }
}
