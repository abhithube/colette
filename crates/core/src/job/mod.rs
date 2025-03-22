use std::{
    fmt::{self, Display},
    str::FromStr,
};

use chrono::{DateTime, Utc};
pub use job_repository::*;
pub use job_service::*;
use serde_json::Value;
use uuid::Uuid;

mod job_repository;
mod job_service;

#[derive(Debug, Clone, bon::Builder)]
pub struct Job {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub job_type: String,
    pub data: Value,
    #[builder(default = Default::default())]
    pub status: JobStatus,
    pub group_id: Option<String>,
    pub message: Option<String>,
    #[builder(default = Utc::now())]
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
pub enum JobStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
}

impl Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
        };

        write!(f, "{}", value)
    }
}

impl FromStr for JobStatus {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Job not found with ID: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
