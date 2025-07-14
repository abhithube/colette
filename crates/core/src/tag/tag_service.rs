use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use super::{Error, Tag, TagCursor, TagParams, TagRepository, TagType};
use crate::pagination::{Paginated, paginate};

pub struct TagService {
    repository: Arc<dyn TagRepository>,
}

impl TagService {
    pub fn new(repository: Arc<dyn TagRepository>) -> Self {
        Self { repository }
    }

    pub async fn list_tags(
        &self,
        query: TagListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<Tag, TagCursor>, Error> {
        let tags = self
            .repository
            .query(TagParams {
                tag_type: query.tag_type,
                user_id: Some(user_id),
                cursor: query.cursor.map(|e| e.title),
                limit: query.limit.map(|e| e + 1),
                with_subscription_count: true,
                ..Default::default()
            })
            .await?;

        if let Some(limit) = query.limit {
            Ok(paginate(tags, limit))
        } else {
            Ok(Paginated {
                items: tags,
                ..Default::default()
            })
        }
    }

    pub async fn get_tag(&self, query: TagGetQuery, user_id: Uuid) -> Result<Tag, Error> {
        let mut tags = self
            .repository
            .query(TagParams {
                ids: Some(vec![query.id]),
                with_subscription_count: query.with_subscription_count,
                with_bookmark_count: query.with_bookmark_count,
                ..Default::default()
            })
            .await?;
        if tags.is_empty() {
            return Err(Error::NotFound(query.id));
        }

        let tag = tags.swap_remove(0);
        if tag.user_id != user_id {
            return Err(Error::Forbidden(query.id));
        }

        Ok(tag)
    }

    pub async fn create_tag(&self, data: TagCreate, user_id: Uuid) -> Result<Tag, Error> {
        let tag = Tag::builder().title(data.title).user_id(user_id).build();

        self.repository.save(&tag).await?;

        Ok(tag)
    }

    pub async fn update_tag(&self, id: Uuid, data: TagUpdate, user_id: Uuid) -> Result<Tag, Error> {
        let Some(mut tag) = self.repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if tag.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        if let Some(title) = data.title {
            tag.title = title;
        }

        tag.updated_at = Utc::now();
        self.repository.save(&tag).await?;

        Ok(tag)
    }

    pub async fn delete_tag(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        let Some(tag) = self.repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if tag.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.repository.delete_by_id(id).await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct TagListQuery {
    pub tag_type: Option<TagType>,
    pub cursor: Option<TagCursor>,
    pub limit: Option<usize>,
    pub with_subscription_count: bool,
    pub with_bookmark_count: bool,
}

#[derive(Debug, Clone)]
pub struct TagGetQuery {
    pub id: Uuid,
    pub with_subscription_count: bool,
    pub with_bookmark_count: bool,
}

#[derive(Debug, Clone)]
pub struct TagCreate {
    pub title: String,
}

#[derive(Debug, Clone, Default)]
pub struct TagUpdate {
    pub title: Option<String>,
}
