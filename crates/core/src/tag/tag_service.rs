use std::sync::Arc;

use uuid::Uuid;

use super::{Error, Tag, TagCursor, TagFindParams, TagRepository};
use crate::{
    pagination::{Paginated, paginate},
    tag::{TagInsertParams, TagUpdateParams},
};

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
            .find(TagFindParams {
                user_id: Some(user_id),
                cursor: query.cursor.map(|e| e.title),
                limit: query.limit.map(|e| e + 1),
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

    pub async fn get_tag(&self, id: Uuid, user_id: Uuid) -> Result<Tag, Error> {
        let mut tags = self
            .repository
            .find(TagFindParams {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if tags.is_empty() {
            return Err(Error::NotFound(id));
        }

        let tag = tags.swap_remove(0);
        if tag.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        Ok(tag)
    }

    pub async fn create_tag(&self, data: TagCreate, user_id: Uuid) -> Result<Tag, Error> {
        let id = self
            .repository
            .insert(TagInsertParams {
                title: data.title,
                user_id,
            })
            .await?;

        self.get_tag(id, user_id).await
    }

    pub async fn update_tag(&self, id: Uuid, data: TagUpdate, user_id: Uuid) -> Result<Tag, Error> {
        let Some(tag) = self.repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if tag.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.repository
            .update(TagUpdateParams {
                id,
                title: data.title,
            })
            .await?;

        self.get_tag(id, user_id).await
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
    pub cursor: Option<TagCursor>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct TagCreate {
    pub title: String,
}

#[derive(Debug, Clone, Default)]
pub struct TagUpdate {
    pub title: Option<String>,
}
