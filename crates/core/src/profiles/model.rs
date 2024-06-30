use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ProfileUpdateData;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub title: String,
    pub image_url: Option<String>,
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

impl From<UpdateProfileDto> for ProfileUpdateData {
    fn from(value: UpdateProfileDto) -> Self {
        Self {
            title: value.title,
            image_url: value.image_url,
        }
    }
}
