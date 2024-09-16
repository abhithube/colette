use std::fmt;

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

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let raw = match self {
            Self::V1 => "1.0",
            Self::V1_1 => "1.1",
            Self::V2 => "2.0",
        };

        write!(f, "{}", raw)
    }
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

impl fmt::Display for OutlineType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let raw = match self {
            Self::Rss => "rss",
        };

        write!(f, "{}", raw)
    }
}
