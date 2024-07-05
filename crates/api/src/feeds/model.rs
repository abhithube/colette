use chrono::{DateTime, Utc};
use colette_core::{feeds::CreateFeed, Feed};
use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(title = "Feed")]
pub struct FeedDto {
    pub id: String,
    #[schema(format = "uri")]
    pub link: String,
    pub title: String,
    #[schema(format = "uri")]
    pub url: Option<String>,
    pub custom_title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub unread_count: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(title = "CreateFeed")]
pub struct CreateFeedDto {
    pub url: Url,
}

impl From<Feed> for FeedDto {
    fn from(value: Feed) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            url: value.url,
            custom_title: value.custom_title,
            created_at: value.created_at,
            updated_at: value.updated_at,
            unread_count: value.unread_count,
        }
    }
}

impl<'a> From<&'a CreateFeedDto> for CreateFeed<'a> {
    fn from(value: &'a CreateFeedDto) -> Self {
        Self {
            url: value.url.as_str(),
        }
    }
}
