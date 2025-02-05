use uuid::Uuid;

use super::{
    Error, Tag, TagType,
    tag_repository::{TagCreateData, TagFindParams, TagRepository, TagUpdateData},
};
use crate::common::{IdParams, NonEmptyString, Paginated};

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
                user_id,
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: tags,
            ..Default::default()
        })
    }

    pub async fn get_tag(&self, id: Uuid, user_id: Uuid) -> Result<Tag, Error> {
        let mut tags = self
            .repository
            .find(TagFindParams {
                id: Some(id),
                user_id,
                ..Default::default()
            })
            .await?;
        if tags.is_empty() {
            return Err(Error::NotFound(id));
        }

        Ok(tags.swap_remove(0))
    }

    pub async fn create_tag(&self, data: TagCreate, user_id: Uuid) -> Result<Tag, Error> {
        let id = self
            .repository
            .create(TagCreateData {
                title: data.title,
                user_id,
            })
            .await?;

        self.get_tag(id, user_id).await
    }

    pub async fn update_tag(&self, id: Uuid, data: TagUpdate, user_id: Uuid) -> Result<Tag, Error> {
        self.repository
            .update(IdParams::new(id, user_id), data.into())
            .await?;

        self.get_tag(id, user_id).await
    }

    pub async fn delete_tag(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        self.repository.delete(IdParams::new(id, user_id)).await
    }
}

#[derive(Clone, Debug, Default)]
pub struct TagListQuery {
    pub tag_type: TagType,
}

impl From<TagUpdate> for TagUpdateData {
    fn from(value: TagUpdate) -> Self {
        Self { title: value.title }
    }
}

#[derive(Clone, Debug)]
pub struct TagCreate {
    pub title: NonEmptyString,
}

#[derive(Clone, Debug, Default)]
pub struct TagUpdate {
    pub title: Option<NonEmptyString>,
}
