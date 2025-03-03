use uuid::Uuid;

use super::{
    Error, FeedEntryFilter, Stream,
    stream_repository::{StreamCreateData, StreamFindParams, StreamRepository, StreamUpdateData},
};
use crate::common::{IdParams, Paginated};

pub struct StreamService {
    repository: Box<dyn StreamRepository>,
}

impl StreamService {
    pub fn new(repository: impl StreamRepository) -> Self {
        Self {
            repository: Box::new(repository),
        }
    }

    pub async fn list_streams(&self, user_id: Uuid) -> Result<Paginated<Stream>, Error> {
        let streams = self
            .repository
            .find_streams(StreamFindParams {
                user_id,
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: streams,
            cursor: None,
        })
    }

    pub async fn get_stream(&self, id: Uuid, user_id: Uuid) -> Result<Stream, Error> {
        let mut streams = self
            .repository
            .find_streams(StreamFindParams {
                id: Some(id),
                user_id,
                ..Default::default()
            })
            .await?;
        if streams.is_empty() {
            return Err(Error::NotFound(id));
        }

        Ok(streams.swap_remove(0))
    }

    pub async fn create_stream(&self, data: StreamCreate, user_id: Uuid) -> Result<Stream, Error> {
        let id = self
            .repository
            .create_stream(StreamCreateData {
                title: data.title,
                filter: data.filter,
                user_id,
            })
            .await?;

        self.get_stream(id, user_id).await
    }

    pub async fn update_stream(
        &self,
        id: Uuid,
        data: StreamUpdate,
        user_id: Uuid,
    ) -> Result<Stream, Error> {
        self.repository
            .update_stream(IdParams::new(id, user_id), data.into())
            .await?;

        self.get_stream(id, user_id).await
    }

    pub async fn delete_stream(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        self.repository
            .delete_stream(IdParams::new(id, user_id))
            .await
    }
}

impl From<StreamUpdate> for StreamUpdateData {
    fn from(value: StreamUpdate) -> Self {
        Self {
            title: value.title,
            filter: value.filter,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StreamCreate {
    pub title: String,
    pub filter: FeedEntryFilter,
}

#[derive(Debug, Clone, Default)]
pub struct StreamUpdate {
    pub title: Option<String>,
    pub filter: Option<FeedEntryFilter>,
}
