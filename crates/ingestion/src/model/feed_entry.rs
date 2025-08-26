use chrono::{DateTime, Utc};
use colette_common::uuid_generate_ctx;
use url::Url;
use uuid::{ContextV7, Uuid};

#[derive(Debug, Clone)]
pub struct FeedEntry {
    id: FeedEntryId,
    link: Url,
    title: String,
    published_at: DateTime<Utc>,
    description: Option<String>,
    author: Option<String>,
    thumbnail_url: Option<Url>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl FeedEntry {
    pub fn new(
        ctx: &ContextV7,
        link: Url,
        title: String,
        published_at: DateTime<Utc>,
        description: Option<String>,
        author: Option<String>,
        thumbnail_url: Option<Url>,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: uuid_generate_ctx(ctx, now).into(),
            link,
            title,
            published_at,
            description,
            author,
            thumbnail_url,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn id(&self) -> FeedEntryId {
        self.id
    }

    pub fn link(&self) -> &Url {
        &self.link
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn set_title(&mut self, value: String) {
        if value != self.title {
            self.title = value;
            self.updated_at = Utc::now();
        }
    }

    pub fn published_at(&self) -> DateTime<Utc> {
        self.published_at
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }

    pub fn thumbnail_url(&self) -> Option<&Url> {
        self.thumbnail_url.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeedEntryId(Uuid);

impl FeedEntryId {
    pub fn new(id: Uuid) -> Self {
        Into::into(id)
    }

    pub fn as_inner(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for FeedEntryId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}
