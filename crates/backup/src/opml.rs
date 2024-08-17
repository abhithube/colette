use colette_core::backup::{self, BackupManager};
use quick_xml::se::Serializer;
use serde::Serialize;
use url::Url;

#[derive(Default)]
pub struct OpmlManager;

impl BackupManager for OpmlManager {
    type T = Opml;

    fn import(&self, raw: &str) -> Result<Self::T, backup::Error> {
        quick_xml::de::from_str::<Opml>(raw).map_err(|_| backup::Error::Deserialize)
    }

    fn export(&self, data: Self::T) -> Result<String, backup::Error> {
        let mut buffer = String::new();
        let mut ser = Serializer::with_root(&mut buffer, Some("opml"))
            .map_err(|_| backup::Error::Serialize)?;
        ser.indent(' ', 2);

        data.serialize(ser)
            .map_err(|_| backup::Error::Deserialize)?;

        let raw = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n".to_owned() + &buffer;

        Ok(raw)
    }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Opml {
    #[serde(default = "default_opml_version", rename = "@version")]
    pub version: String,
    pub head: OpmlHead,
    pub body: OpmlBody,
}

fn default_opml_version() -> String {
    "2.0".to_owned()
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct OpmlHead {
    #[serde(default = "default_head_title")]
    pub title: String,
}

fn default_head_title() -> String {
    "Feeds".to_owned()
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct OpmlBody {
    #[serde(rename = "outline")]
    pub outlines: Vec<OpmlOutline>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OpmlOutline {
    #[serde(rename = "@type")]
    pub outline_type: Option<OpmlOutlineType>,
    #[serde(rename = "@text")]
    pub text: String,
    #[serde(rename = "@xmlUrl")]
    pub xml_url: Option<Url>,
    #[serde(rename = "@title")]
    pub title: Option<String>,
    #[serde(rename = "@htmlUrl")]
    pub html_url: Option<Url>,
    #[serde(rename = "outline")]
    pub children: Option<Vec<OpmlOutline>>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OpmlOutlineType {
    #[default]
    Rss,
    Atom,
}
