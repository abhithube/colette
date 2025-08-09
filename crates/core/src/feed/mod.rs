use std::{
    fmt::{self, Display},
    str::FromStr,
};

use chrono::{DateTime, Utc};
pub use detect_feeds_handler::*;
pub use feed_repository::*;
pub use get_feed_handler::*;
pub use list_feeds_handler::*;
pub use refresh_feed_handler::*;
use url::Url;
use uuid::Uuid;

use crate::pagination::Cursor;

mod detect_feeds_handler;
mod feed_repository;
mod get_feed_handler;
mod list_feeds_handler;
mod refresh_feed_handler;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Feed {
    pub id: Uuid,
    pub source_url: Url,
    pub link: Url,
    pub title: String,
    pub description: Option<String>,
    #[serde(skip_serializing, default = "default_refresh_interval_min")]
    pub refresh_interval_min: u32,
    #[serde(skip_serializing, default = "FeedStatus::default")]
    pub status: FeedStatus,
    #[serde(skip_serializing)]
    pub refreshed_at: Option<DateTime<Utc>>,
    pub is_custom: bool,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename = "lowercase")]
pub enum FeedStatus {
    #[default]
    Pending,
    Healthy,
    Refreshing,
    Failed,
}

impl Display for FeedStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Pending => "pending",
            Self::Healthy => "healthy",
            Self::Refreshing => "refreshing",
            Self::Failed => "failed",
        };

        write!(f, "{value}")
    }
}

impl FromStr for FeedStatus {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "healthy" => Ok(Self::Healthy),
            "refreshing" => Ok(Self::Refreshing),
            "failed" => Ok(Self::Failed),
            _ => unreachable!(),
        }
    }
}

fn default_refresh_interval_min() -> u32 {
    60
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeedCursor {
    pub source_url: Url,
}

impl Cursor for Feed {
    type Data = FeedCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            source_url: self.source_url.clone(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeedDetected {
    pub url: Url,
    pub title: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScrapeFeedJobData {
    pub url: Url,
}
