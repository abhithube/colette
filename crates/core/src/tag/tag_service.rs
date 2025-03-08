use uuid::Uuid;

use super::{
    Error, Tag, TagCreateParams, TagDeleteParams, TagFindByIdsParams, TagFindParams, TagRepository,
    TagType, TagUpdateParams,
};
use crate::common::{Paginated, TransactionManager};

pub struct TagService {
    repository: Box<dyn TagRepository>,
    tx_manager: Box<dyn TransactionManager>,
}

impl TagService {
    pub fn new(repository: impl TagRepository, tx_manager: impl TransactionManager) -> Self {
        Self {
            repository: Box::new(repository),
            tx_manager: Box::new(tx_manager),
        }
    }

    pub async fn list_tags(
        &self,
        query: TagListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<Tag>, Error> {
        let tags = self
            .repository
            .find_tags(TagFindParams {
                tag_type: query.tag_type,
                user_id: Some(user_id),
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
            .find_tags(TagFindParams {
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
        let id = Uuid::new_v4();

        self.repository
            .create_tag(TagCreateParams {
                id,
                title: data.title,
                user_id,
            })
            .await?;

        self.get_tag(id, user_id).await
    }

    pub async fn update_tag(&self, id: Uuid, data: TagUpdate, user_id: Uuid) -> Result<Tag, Error> {
        let tx = self.tx_manager.begin().await?;

        let mut tags = self
            .repository
            .find_tags_by_ids(&*tx, TagFindByIdsParams { ids: vec![id] })
            .await?;
        if tags.is_empty() {
            return Err(Error::NotFound(id));
        }

        let tag = tags.swap_remove(0);
        if tag.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.repository
            .update_tag(
                &*tx,
                TagUpdateParams {
                    id,
                    title: data.title,
                },
            )
            .await?;

        tx.commit().await?;

        self.get_tag(id, user_id).await
    }

    pub async fn delete_tag(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        let tx = self.tx_manager.begin().await?;

        let mut tags = self
            .repository
            .find_tags_by_ids(&*tx, TagFindByIdsParams { ids: vec![id] })
            .await?;
        if tags.is_empty() {
            return Err(Error::NotFound(id));
        }

        let tag = tags.swap_remove(0);
        if tag.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.repository
            .delete_tag(&*tx, TagDeleteParams { id })
            .await?;

        tx.commit().await?;

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
