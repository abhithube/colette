pub use reader::from_reader;
pub use writer::to_writer;

mod reader;
mod writer;

#[derive(Clone, Debug, Default)]
pub struct Opml {
    pub version: Version,
    pub head: Head,
    pub body: Body,
}

#[derive(Clone, Debug, Default)]
pub enum Version {
    V1,
    V1_1,
    #[default]
    V2,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug, Default)]
pub struct Body {
    pub outlines: Vec<Outline>,
}

#[derive(Clone, Debug, Default)]
pub struct Outline {
    pub r#type: Option<OutlineType>,
    pub text: String,
    pub xml_url: Option<String>,
    pub title: Option<String>,
    pub html_url: Option<String>,
    pub outline: Option<Vec<Outline>>,
}

#[derive(Clone, Debug, Default)]
pub enum OutlineType {
    #[default]
    Rss,
}
