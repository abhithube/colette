use std::{
    fmt::{self, Display},
    str::FromStr,
};

use chrono::{DateTime, Utc};
pub use create_job_handler::*;
pub use get_job_handler::*;
pub use job_repository::*;
use serde_json::Value;
pub use update_job_handler::*;
use uuid::Uuid;

mod create_job_handler;
mod get_job_handler;
mod job_repository;
mod update_job_handler;

#[derive(Debug, Clone)]
pub struct Job {
    pub id: Uuid,
    pub job_type: String,
    pub data: Value,
    pub status: JobStatus,
    pub group_identifier: Option<String>,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, PartialEq)]
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

        write!(f, "{value}")
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
