use chrono::{DateTime, Utc};

use super::ProfileUpdateData;

#[derive(Debug)]
pub struct Profile {
    pub id: String,
    pub title: String,
    pub image_url: Option<String>,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CreateProfile {
    pub title: String,
    pub image_url: Option<String>,
}

#[derive(Debug)]
pub struct UpdateProfile {
    pub title: Option<String>,
    pub image_url: Option<String>,
}

impl From<UpdateProfile> for ProfileUpdateData {
    fn from(value: UpdateProfile) -> Self {
        Self {
            title: value.title,
            image_url: value.image_url,
        }
    }
}
