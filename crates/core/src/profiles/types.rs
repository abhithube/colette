use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub title: String,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct ProfileFindManyParams {
    pub user_id: String,
}

pub struct ProfileFindByIdParams {
    pub id: String,
    pub user_id: String,
}

pub enum ProfileFindOneParams {
    ById(ProfileFindByIdParams),
    Default { user_id: String },
}

pub struct ProfileCreateData {
    pub title: String,
    pub image_url: Option<String>,
    pub user_id: String,
}

pub struct ProfileUpdateData {
    pub title: Option<String>,
    pub image_url: Option<String>,
}
