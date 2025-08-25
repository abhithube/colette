use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Feed {
    pub id: FeedId,
    pub source_url: Url,
    pub link: Url,
    pub title: String,
    pub description: Option<String>,
    pub refresh_interval_min: u32,
    pub status: FeedStatus,
    pub refreshed_at: Option<DateTime<Utc>>,
    pub is_custom: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeedId(Uuid);

impl FeedId {
    pub fn new(id: Uuid) -> Self {
        Into::into(id)
    }

    pub fn as_inner(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for FeedId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Default)]
pub enum FeedStatus {
    #[default]
    Pending,
    Healthy,
    Refreshing,
    Failed,
}

impl fmt::Display for FeedStatus {
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
    type Err = FeedError;

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

#[derive(Debug, Clone)]
pub struct FeedDetected {
    pub url: Url,
    pub title: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScrapeFeedJobData {
    pub url: Url,
}

#[derive(Debug, thiserror::Error)]
pub enum FeedError {
    #[error("feed not found with ID: {0}")]
    NotFound(Uuid),
}
