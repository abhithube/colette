use chrono::Utc;
use uuid::Uuid;

use super::{Error, Tag, TagFindParams, TagRepository, TagType, TagUpsertType};
use crate::common::Paginated;

pub struct TagService {
    repository: Box<dyn TagRepository>,
}

impl TagService {
    pub fn new(repository: impl TagRepository) -> Self {
        Self {
            repository: Box::new(repository),
        }
    }

    pub async fn list_tags(
        &self,
        query: TagListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<Tag>, Error> {
        let tags = self
            .repository
            .find(TagFindParams {
                tag_type: query.tag_type,
                user_id: Some(user_id),
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: tags,
            cursor: None,
        })
    }

    pub async fn get_tag(&self, id: Uuid, user_id: Uuid) -> Result<Tag, Error> {
        let mut tags = self
            .repository
            .find(TagFindParams {
                ids: Some(vec![id]),
                ..Default::default()
            })
            .await?;
        if tags.is_empty() {
            return Err(Error::NotFound(id));
        }

        let tag = tags.swap_remove(0);
        if tag.user_id != user_id {
            return Err(Error::Forbidden(tag.id));
        }

        Ok(tag)
    }

    pub async fn create_tag(&self, data: TagCreate, user_id: Uuid) -> Result<Tag, Error> {
        let tag = Tag::builder().title(data.title).user_id(user_id).build();

        self.repository.save(&tag, None).await?;

        Ok(tag)
    }

    pub async fn update_tag(&self, id: Uuid, data: TagUpdate, user_id: Uuid) -> Result<Tag, Error> {
        let mut tags = self.repository.find_by_ids(vec![id]).await?;
        if tags.is_empty() {
            return Err(Error::NotFound(id));
        }

        let mut tag = tags.swap_remove(0);
        if tag.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        if let Some(title) = data.title {
            tag.title = title;
        }

        tag.updated_at = Utc::now();
        self.repository.save(&tag, Some(TagUpsertType::Id)).await?;

        Ok(tag)
    }

    pub async fn delete_tag(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        let mut tags = self.repository.find_by_ids(vec![id]).await?;
        if tags.is_empty() {
            return Err(Error::NotFound(id));
        }

        let tag = tags.swap_remove(0);
        if tag.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.repository.delete_by_id(id).await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct TagListQuery {
    pub tag_type: TagType,
}

#[derive(Debug, Clone)]
pub struct TagCreate {
    pub title: String,
}

#[derive(Debug, Clone, Default)]
pub struct TagUpdate {
    pub title: Option<String>,
}
