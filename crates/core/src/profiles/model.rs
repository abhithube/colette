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
pub struct CreateProfileDto {
    pub title: String,
    pub image_url: Option<String>,
}

#[derive(Debug)]
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
