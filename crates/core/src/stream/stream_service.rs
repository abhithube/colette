use chrono::Utc;
use uuid::Uuid;

use super::{Error, Stream, StreamParams, StreamRepository, SubscriptionEntryFilter};
use crate::common::Paginated;

pub struct StreamService {
    repository: Box<dyn StreamRepository>,
}

impl StreamService {
    pub fn new(repository: impl StreamRepository) -> Self {
        Self {
            repository: Box::new(repository),
        }
    }

    pub async fn list_streams(&self, user_id: String) -> Result<Paginated<Stream>, Error> {
        let streams = self
            .repository
            .query(StreamParams {
                user_id: Some(user_id),
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: streams,
            cursor: None,
        })
    }

    pub async fn get_stream(&self, id: Uuid, user_id: String) -> Result<Stream, Error> {
        let mut streams = self
            .repository
            .query(StreamParams {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if streams.is_empty() {
            return Err(Error::NotFound(id));
        }

        let stream = streams.swap_remove(0);
        if stream.user_id != user_id {
            return Err(Error::Forbidden(stream.id));
        }

        Ok(stream)
    }

    pub async fn create_stream(
        &self,
        data: StreamCreate,
        user_id: String,
    ) -> Result<Stream, Error> {
        let stream = Stream::builder()
            .title(data.title)
            .filter(data.filter)
            .user_id(user_id)
            .build();

        self.repository.save(&stream).await?;

        Ok(stream)
    }

    pub async fn update_stream(
        &self,
        id: Uuid,
        data: StreamUpdate,
        user_id: String,
    ) -> Result<Stream, Error> {
        let Some(mut stream) = self.repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if stream.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        if let Some(title) = data.title {
            stream.title = title;
        }
        if let Some(filter) = data.filter {
            stream.filter = filter;
        }

        stream.updated_at = Utc::now();
        self.repository.save(&stream).await?;

        Ok(stream)
    }

    pub async fn delete_stream(&self, id: Uuid, user_id: String) -> Result<(), Error> {
        let Some(stream) = self.repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if stream.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.repository.delete_by_id(id).await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct StreamCreate {
    pub title: String,
    pub filter: SubscriptionEntryFilter,
}

#[derive(Debug, Clone, Default)]
pub struct StreamUpdate {
    pub title: Option<String>,
    pub filter: Option<SubscriptionEntryFilter>,
}
