use chrono::Utc;
use uuid::Uuid;

use super::{Error, Tag, TagParams, TagRepository, TagType};
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
        user_id: String,
    ) -> Result<Paginated<Tag>, Error> {
        let tags = self
            .repository
            .query(TagParams {
                tag_type: query.tag_type,
                user_id: Some(user_id),
                with_feed_count: true,
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: tags,
            cursor: None,
        })
    }

    pub async fn get_tag(&self, query: TagGetQuery, user_id: String) -> Result<Tag, Error> {
        let mut tags = self
            .repository
            .query(TagParams {
                ids: Some(vec![query.id]),
                with_feed_count: query.with_feed_count,
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

    pub async fn create_tag(&self, data: TagCreate, user_id: String) -> Result<Tag, Error> {
        let tag = Tag::builder().title(data.title).user_id(user_id).build();

        self.repository.save(&tag).await?;

        Ok(tag)
    }

    pub async fn update_tag(
        &self,
        id: Uuid,
        data: TagUpdate,
        user_id: String,
    ) -> Result<Tag, Error> {
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

    pub async fn delete_tag(&self, id: Uuid, user_id: String) -> Result<(), Error> {
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
    pub with_feed_count: bool,
    pub with_bookmark_count: bool,
}

#[derive(Debug, Clone)]
pub struct TagGetQuery {
    pub id: Uuid,
    pub with_feed_count: bool,
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
