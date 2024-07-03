use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ProfileUpdateData;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub title: String,
    pub image_url: Option<String>,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProfileDto {
    pub title: String,
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileDto {
    pub title: Option<String>,
    pub image_url: Option<String>,
}

impl<'a> From<&'a UpdateProfileDto> for ProfileUpdateData<'a> {
    fn from(value: &'a UpdateProfileDto) -> Self {
        Self {
            title: value.title.as_deref(),
            image_url: value.image_url.as_deref(),
        }
    }
}
