use chrono::{DateTime, Utc};
use colette_authentication::UserId;
use colette_common::uuid_generate_ts;
use url::Url;
use uuid::Uuid;

use crate::{
    filter::{BooleanOp, DateOp, NumberOp, TextOp},
    tag::TagId,
};

pub const BOOKMARK_TITLE_MAX_LENGTH: usize = 100;
pub const BOOKMARK_AUTHOR_MAX_LENGTH: usize = 50;
pub const BOOKMARK_TAG_MAX_COUNT: usize = 20;

#[derive(Debug, Clone)]
pub struct Bookmark {
    id: BookmarkId,
    link: Url,
    title: BookmarkTitle,
    thumbnail_url: Option<Url>,
    published_at: Option<DateTime<Utc>>,
    author: Option<BookmarkAuthor>,
    tags: Vec<TagId>,
    user_id: UserId,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Bookmark {
    pub fn new(
        link: Url,
        title: BookmarkTitle,
        thumbnail_url: Option<Url>,
        published_at: Option<DateTime<Utc>>,
        author: Option<BookmarkAuthor>,
        user_id: UserId,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: uuid_generate_ts(now).into(),
            link,
            title,
            thumbnail_url,
            published_at,
            author,
            tags: Vec::new(),
            user_id,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn id(&self) -> BookmarkId {
        self.id
    }

    pub fn link(&self) -> &Url {
        &self.link
    }

    pub fn title(&self) -> &BookmarkTitle {
        &self.title
    }

    pub fn set_title(&mut self, value: BookmarkTitle) {
        if value != self.title {
            self.title = value;
            self.updated_at = Utc::now();
        }
    }

    pub fn thumbnail_url(&self) -> Option<&Url> {
        self.thumbnail_url.as_ref()
    }

    pub fn set_thumbnail_url(&mut self, value: Url) {
        if self.thumbnail_url.as_ref().is_none_or(|e| &value != e) {
            self.thumbnail_url = Some(value);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_thumbnail_url(&mut self) {
        if self.thumbnail_url.is_some() {
            self.thumbnail_url = None;
            self.updated_at = Utc::now();
        }
    }

    pub fn published_at(&self) -> Option<DateTime<Utc>> {
        self.published_at
    }

    pub fn set_published_at(&mut self, value: DateTime<Utc>) {
        if self.published_at.as_ref().is_none_or(|e| &value != e) {
            self.published_at = Some(value);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_published_at(&mut self) {
        if self.published_at.is_some() {
            self.published_at = None;
            self.updated_at = Utc::now();
        }
    }

    pub fn author(&self) -> Option<&BookmarkAuthor> {
        self.author.as_ref()
    }

    pub fn set_author(&mut self, value: BookmarkAuthor) {
        if self.author.as_ref().is_none_or(|e| &value != e) {
            self.author = Some(value);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_author(&mut self) {
        if self.author.is_some() {
            self.author = None;
            self.updated_at = Utc::now();
        }
    }

    pub fn tags(&self) -> &[TagId] {
        &self.tags
    }

    pub fn set_tags(&mut self, value: Vec<TagId>) -> Result<(), BookmarkError> {
        if value.len() > BOOKMARK_TAG_MAX_COUNT {
            return Err(BookmarkError::TooManyTags);
        }

        self.tags = value;

        Ok(())
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_unchecked(
        id: Uuid,
        link: Url,
        title: String,
        thumbnail_url: Option<Url>,
        published_at: Option<DateTime<Utc>>,
        author: Option<String>,
        tags: Vec<Uuid>,
        user_id: Uuid,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: BookmarkId(id),
            link,
            title: BookmarkTitle(title),
            thumbnail_url,
            published_at,
            author: author.map(BookmarkAuthor),
            tags: tags.into_iter().map(Into::into).collect(),
            user_id: user_id.into(),
            created_at,
            updated_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct BookmarkId(Uuid);

impl BookmarkId {
    pub fn new(id: Uuid) -> Self {
        Into::into(id)
    }

    pub fn as_inner(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for BookmarkId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookmarkTitle(String);

impl BookmarkTitle {
    pub fn new(value: String) -> Result<Self, BookmarkError> {
        if value.is_empty() || value.len() > BOOKMARK_TITLE_MAX_LENGTH {
            return Err(BookmarkError::InvalidTitleLength);
        }

        Ok(Self(value))
    }

    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookmarkAuthor(String);

impl BookmarkAuthor {
    pub fn new(value: String) -> Result<Self, BookmarkError> {
        if value.is_empty() || value.len() > BOOKMARK_AUTHOR_MAX_LENGTH {
            return Err(BookmarkError::InvalidAuthorLength);
        }

        Ok(Self(value))
    }

    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScrapeBookmarkJobData {
    pub url: Url,
    pub user_id: UserId,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchiveThumbnailJobData {
    pub operation: ThumbnailOperation,
    pub archived_path: Option<String>,
    pub bookmark_id: BookmarkId,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ThumbnailOperation {
    Upload(Url),
    Delete,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BookmarkFilter {
    Text {
        field: BookmarkTextField,
        op: TextOp,
    },
    Number {
        field: BookmarkNumberField,
        op: NumberOp,
    },
    Boolean {
        field: BookmarkBooleanField,
        op: BooleanOp,
    },
    Date {
        field: BookmarkDateField,
        op: DateOp,
    },

    And(Vec<BookmarkFilter>),
    Or(Vec<BookmarkFilter>),
    Not(Box<BookmarkFilter>),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BookmarkTextField {
    Link,
    Title,
    Author,
    Tag,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BookmarkNumberField {}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BookmarkBooleanField {}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BookmarkDateField {
    PublishedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, thiserror::Error)]
pub enum BookmarkError {
    #[error("title must be between 1 and {BOOKMARK_TITLE_MAX_LENGTH} characters long")]
    InvalidTitleLength,

    #[error("author must be between 1 and {BOOKMARK_AUTHOR_MAX_LENGTH} characters long")]
    InvalidAuthorLength,

    #[error("bookmark already exists with URL: {0}")]
    Conflict(Url),

    #[error("bookmark not found with ID: {0}")]
    NotFound(Uuid),

    #[error("bookmark cannot have more than {BOOKMARK_TAG_MAX_COUNT} tags")]
    TooManyTags,
}
