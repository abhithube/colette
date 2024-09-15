pub use reader::from_reader;
pub use writer::to_writer;

mod reader;
mod writer;

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Opml {
    #[serde(rename = "@version")]
    pub version: Version,
    pub head: Head,
    pub body: Body,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Version {
    #[serde(rename = "1.0")]
    V1,
    #[serde(rename = "1.1")]
    V1_1,
    #[default]
    #[serde(rename = "2.0")]
    V2,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Head {
    pub title: String,
}

impl Default for Head {
    fn default() -> Self {
        Self {
            title: "Feeds".to_owned(),
        }
    }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
    #[serde(rename = "outline")]
    pub outlines: Vec<Outline>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Outline {
    #[serde(rename = "@type")]
    pub r#type: Option<OutlineType>,
    #[serde(rename = "@text")]
    pub text: String,
    #[serde(rename = "@xmlUrl")]
    pub xml_url: Option<String>,
    #[serde(rename = "@title")]
    pub title: Option<String>,
    #[serde(rename = "@htmlUrl")]
    pub html_url: Option<String>,
    #[serde(rename = "outline")]
    pub outline: Option<Vec<Outline>>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OutlineType {
    #[default]
    Rss,
}
