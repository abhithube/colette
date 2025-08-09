use colette_core::job::{CreateJobError, GetJobError, UpdateJobError};

pub mod archive_thumbnail;
pub mod import_bookmarks;
pub mod refresh_feeds;
pub mod scrape_bookmark;
pub mod scrape_feed;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Job(#[from] JobError),

    #[error(transparent)]
    Queue(#[from] colette_queue::Error),

    #[error(transparent)]
    Serialize(#[from] serde_json::Error),

    #[error("service error: {0}")]
    Service(String),
}

#[derive(Debug, thiserror::Error)]
pub enum JobError {
    #[error(transparent)]
    GetJob(#[from] GetJobError),

    #[error(transparent)]
    CreateJob(#[from] CreateJobError),

    #[error(transparent)]
    UpdateJob(#[from] UpdateJobError),
}
