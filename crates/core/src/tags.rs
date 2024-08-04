use std::sync::Arc;

use uuid::Uuid;

use crate::common::{FindOneParams, Paginated, Session};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Tag {
    pub id: Uuid,
    pub title: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct CreateTag {
    pub title: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct UpdateTag {
    pub title: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct ListTagsParams {
    pub tag_type: TagType,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub enum TagType {
    All,
    Bookmarks,
    Feeds,
}

#[async_trait::async_trait]
pub trait TagsRepository: Send + Sync {
    async fn find_many_tags(&self, params: TagsFindManyParams) -> Result<Vec<Tag>, Error>;

    async fn find_one_tag(&self, params: FindOneParams) -> Result<Tag, Error>;

    async fn create_tag(&self, data: TagsCreateData) -> Result<Tag, Error>;

    async fn update_tag(&self, params: FindOneParams, data: TagsUpdateData) -> Result<Tag, Error>;

    async fn delete_tag(&self, params: FindOneParams) -> Result<(), Error>;
}

pub struct TagsService {
    repo: Arc<dyn TagsRepository>,
}

impl TagsService {
    pub fn new(repo: Arc<dyn TagsRepository>) -> Self {
        Self { repo }
    }

    pub async fn list(
        &self,
        params: ListTagsParams,
        session: Session,
    ) -> Result<Paginated<Tag>, Error> {
        let tags = self
            .repo
            .find_many_tags(TagsFindManyParams {
                profile_id: session.profile_id,
                tag_type: params.tag_type,
            })
            .await?;

        Ok(Paginated::<Tag> {
            has_more: false,
            data: tags,
        })
    }

    pub async fn get(&self, id: Uuid, session: Session) -> Result<Tag, Error> {
        self.repo
            .find_one_tag(FindOneParams {
                id,
                profile_id: session.profile_id,
            })
            .await
    }

    pub async fn create(&self, data: CreateTag, session: Session) -> Result<Tag, Error> {
        self.repo
            .create_tag(TagsCreateData {
                title: data.title,
                profile_id: session.profile_id,
            })
            .await
    }

    pub async fn update(&self, id: Uuid, data: UpdateTag, session: Session) -> Result<Tag, Error> {
        self.repo
            .update_tag(
                FindOneParams {
                    id,
                    profile_id: session.profile_id,
                },
                data.into(),
            )
            .await
    }

    pub async fn delete(&self, id: Uuid, session: Session) -> Result<(), Error> {
        self.repo
            .delete_tag(FindOneParams {
                id,
                profile_id: session.profile_id,
            })
            .await
    }
}

#[derive(Clone, Debug)]
pub struct TagsFindManyParams {
    pub profile_id: Uuid,
    pub tag_type: TagType,
}

#[derive(Clone, Debug)]
pub struct TagsCreateData {
    pub title: String,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct TagsUpdateData {
    pub title: Option<String>,
}

impl From<UpdateTag> for TagsUpdateData {
    fn from(value: UpdateTag) -> Self {
        Self { title: value.title }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("tag not found with id: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
