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
pub struct CreateProfile<'a> {
    pub title: &'a str,
    pub image_url: Option<&'a str>,
}

#[derive(Debug)]
pub struct UpdateProfile<'a> {
    pub title: Option<&'a str>,
    pub image_url: Option<&'a str>,
}

impl<'a> From<UpdateProfile<'a>> for ProfileUpdateData<'a> {
    fn from(value: UpdateProfile<'a>) -> Self {
        Self {
            title: value.title,
            image_url: value.image_url,
        }
    }
}
