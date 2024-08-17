use colette_core::{
    backup::{self, BackupManager},
    feed::BackupFeed,
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
            body: OpmlBody {
                outlines: data.into_iter().map(OpmlOutline::from).collect(),
            },
            ..Default::default()
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

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
struct Opml {
    #[serde(default = "default_opml_version", rename = "@version")]
    version: String,
    head: OpmlHead,
    body: OpmlBody,
}

fn default_opml_version() -> String {
    "2.0".to_owned()
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
struct OpmlHead {
    #[serde(default = "default_head_title")]
    title: String,
}

fn default_head_title() -> String {
    "Feeds".to_owned()
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
struct OpmlBody {
    #[serde(rename = "outline")]
    outlines: Vec<OpmlOutline>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct OpmlOutline {
    #[serde(rename = "@type")]
    outline_type: Option<OpmlOutlineType>,
    #[serde(rename = "@text")]
    text: String,
    #[serde(rename = "@xmlUrl")]
    xml_url: Option<Url>,
    #[serde(rename = "@title")]
    title: Option<String>,
    #[serde(rename = "@htmlUrl")]
    html_url: Option<Url>,
    #[serde(rename = "outline")]
    children: Option<Vec<OpmlOutline>>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum OpmlOutlineType {
    #[default]
    Rss,
    Atom,
}

impl From<OpmlOutline> for BackupFeed {
    fn from(value: OpmlOutline) -> Self {
        Self {
            title: value.title.unwrap_or(value.text),
            xml_url: value.xml_url.unwrap(),
            html_url: value.html_url,
        }
    }
}

impl From<BackupFeed> for OpmlOutline {
    fn from(value: BackupFeed) -> Self {
        Self {
            text: value.title.clone(),
            title: Some(value.title),
            outline_type: Some(OpmlOutlineType::default()),
            xml_url: Some(value.xml_url),
            html_url: value.html_url,
            children: None,
        }
    }
}

fn extract_feeds(outlines: Vec<OpmlOutline>, feeds: &mut Vec<BackupFeed>) {
    for outline in outlines {
        if let Some(children) = outline.children {
            extract_feeds(children, feeds);
        } else {
            feeds.push(BackupFeed::from(outline));
        }
    }
}
