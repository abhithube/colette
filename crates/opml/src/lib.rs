use std::{fmt, str::FromStr};

use quick_xml::events::attributes::AttrError;
pub use reader::from_reader;
pub use writer::to_writer;

mod reader;
mod writer;

#[derive(Debug, Clone, Default)]
pub struct Opml {
    pub version: Version,
    pub head: Head,
    pub body: Body,
}

#[derive(Debug, Clone, Default)]
pub enum Version {
    V1,
    V1_1,
    #[default]
    V2,
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1.0" => Ok(Self::V1),
            "1.1" => Ok(Self::V1_1),
            "2.0" => Ok(Self::V2),
            _ => Err(Error::Parse(ParseError::Version)),
        }
    }
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Default)]
pub struct Body {
    pub outlines: Vec<Outline>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Outline {
    pub r#type: Option<OutlineType>,
    pub text: String,
    pub xml_url: Option<String>,
    pub title: Option<String>,
    pub html_url: Option<String>,
    pub outline: Option<Vec<Outline>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum OutlineType {
    #[default]
    Rss,
}

impl FromStr for OutlineType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rss" => Ok(Self::Rss),
            _ => Err(Error::Parse(ParseError::OutlineType)),
        }
    }
}

impl fmt::Display for OutlineType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let raw = match self {
            Self::Rss => "rss",
        };

        write!(f, "{}", raw)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Xml(#[from] quick_xml::Error),

    #[error(transparent)]
    Parse(#[from] ParseError),
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("OPML version not supported")]
    Version,

    #[error("outline type not supported")]
    OutlineType,

    #[error(transparent)]
    Attribute(#[from] AttrError),
}
