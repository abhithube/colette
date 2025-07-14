use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use super::{Error, Stream, StreamCursor, StreamParams, StreamRepository, SubscriptionEntryFilter};
use crate::pagination::{Paginated, paginate};

pub struct StreamService {
    repository: Arc<dyn StreamRepository>,
}

impl StreamService {
    pub fn new(repository: Arc<dyn StreamRepository>) -> Self {
        Self { repository }
    }

    pub async fn list_streams(
        &self,
        query: StreamListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<Stream, StreamCursor>, Error> {
        let streams = self
            .repository
            .query(StreamParams {
                user_id: Some(user_id),
                cursor: query.cursor.map(|e| e.title),
                limit: query.limit.map(|e| e + 1),
                ..Default::default()
            })
            .await?;

        if let Some(limit) = query.limit {
            Ok(paginate(streams, limit))
        } else {
            Ok(Paginated {
                items: streams,
                ..Default::default()
            })
        }
    }

    pub async fn get_stream(&self, id: Uuid, user_id: Uuid) -> Result<Stream, Error> {
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

    pub async fn create_stream(&self, data: StreamCreate, user_id: Uuid) -> Result<Stream, Error> {
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
        user_id: Uuid,
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

    pub async fn delete_stream(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
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

#[derive(Debug, Clone, Default)]
pub struct StreamListQuery {
    pub cursor: Option<StreamCursor>,
    pub limit: Option<usize>,
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
