use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use colette_common::uuid_generate_ts;
use url::Url;
use uuid::Uuid;

pub const DEFAULT_INTERVAL: u32 = 60;

#[derive(Debug, Clone)]
pub struct Feed {
    id: FeedId,
    source_url: Url,
    link: Url,
    title: String,
    description: Option<String>,
    is_custom: bool,
    status: FeedStatus,
    refresh_interval_min: u32,
    last_refreshed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Feed {
    pub fn new(
        source_url: Url,
        link: Url,
        title: String,
        description: Option<String>,
        is_custom: bool,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: uuid_generate_ts(now).into(),
            source_url,
            link,
            title,
            description,
            refresh_interval_min: DEFAULT_INTERVAL,
            status: FeedStatus::default(),
            last_refreshed_at: None,
            is_custom,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn id(&self) -> FeedId {
        self.id
    }

    pub fn source_url(&self) -> &Url {
        &self.source_url
    }

    pub fn link(&self) -> &Url {
        &self.link
    }

    pub fn set_link(&mut self, value: Url) {
        if value != self.link {
            self.link = value;

            let now = Utc::now();
            self.last_refreshed_at = Some(now);
            self.updated_at = now;
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn set_title(&mut self, value: String) {
        if value != self.title {
            self.title = value;

            let now = Utc::now();
            self.last_refreshed_at = Some(now);
            self.updated_at = now;
        }
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn set_description(&mut self, value: String) {
        if self.description.as_ref().is_none_or(|e| &value != e) {
            self.description = Some(value);

            let now = Utc::now();
            self.last_refreshed_at = Some(now);
            self.updated_at = now;
        }
    }

    pub fn remove_description(&mut self) {
        if self.description.is_some() {
            self.description = None;

            let now = Utc::now();
            self.last_refreshed_at = Some(now);
            self.updated_at = now;
        }
    }

    pub fn is_custom(&self) -> bool {
        self.is_custom
    }

    pub fn status(&self) -> &FeedStatus {
        &self.status
    }

    pub fn set_status(&mut self, value: FeedStatus) {
        if value != self.status {
            self.status = value;

            let now = Utc::now();
            self.last_refreshed_at = Some(now);
            self.updated_at = now;
        }
    }

    pub fn refresh_interval_min(&self) -> u32 {
        self.refresh_interval_min
    }

    pub fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
        self.last_refreshed_at
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_unchecked(
        id: Uuid,
        source_url: Url,
        link: Url,
        title: String,
        description: Option<String>,
        is_custom: bool,
        status: FeedStatus,
        refresh_interval_min: u32,
        last_refreshed_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: FeedId(id),
            source_url,
            link,
            title,
            description,
            is_custom,
            status,
            refresh_interval_min,
            last_refreshed_at,
            created_at,
            updated_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum FeedStatus {
    #[default]
    Healthy,
    Refreshing,
    Failing,
    Disabled,
}

impl fmt::Display for FeedStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Healthy => "healthy",
            Self::Refreshing => "refreshing",
            Self::Failing => "failing",
            Self::Disabled => "disabled",
        };

        write!(f, "{value}")
    }
}

impl FromStr for FeedStatus {
    type Err = FeedError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "healthy" => Ok(Self::Healthy),
            "refreshing" => Ok(Self::Refreshing),
            "failing" => Ok(Self::Failing),
            "disabled" => Ok(Self::Disabled),
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
    pub feed_id: FeedId,
    pub source_url: Url,
}

#[derive(Debug, thiserror::Error)]
pub enum FeedError {
    #[error("feed not found with ID: {0}")]
    NotFound(Uuid),
}
