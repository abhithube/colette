use uuid::Uuid;

use super::{
    Error, Stream, StreamCreateParams, StreamDeleteParams, StreamFindByIdParams, StreamFindParams,
    StreamRepository, StreamUpdateParams, SubscriptionEntryFilter,
};
use crate::common::{Paginated, TransactionManager};

pub struct StreamService {
    repository: Box<dyn StreamRepository>,
    tx_manager: Box<dyn TransactionManager>,
}

impl StreamService {
    pub fn new(repository: impl StreamRepository, tx_manager: impl TransactionManager) -> Self {
        Self {
            repository: Box::new(repository),
            tx_manager: Box::new(tx_manager),
        }
    }

    pub async fn list_streams(&self, user_id: Uuid) -> Result<Paginated<Stream>, Error> {
        let streams = self
            .repository
            .find_streams(StreamFindParams {
                user_id: Some(user_id),
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
        let id = Uuid::new_v4();

        self.repository
            .create_stream(StreamCreateParams {
                id,
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
        let tx = self.tx_manager.begin().await?;

        let stream = self
            .repository
            .find_stream_by_id(&*tx, StreamFindByIdParams { id })
            .await?;
        if stream.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.repository
            .update_stream(
                &*tx,
                StreamUpdateParams {
                    id,
                    title: data.title,
                    filter: data.filter,
                },
            )
            .await?;

        tx.commit().await?;

        self.get_stream(id, user_id).await
    }

    pub async fn delete_stream(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        let tx = self.tx_manager.begin().await?;

        let stream = self
            .repository
            .find_stream_by_id(&*tx, StreamFindByIdParams { id })
            .await?;
        if stream.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.repository
            .delete_stream(&*tx, StreamDeleteParams { id })
            .await?;

        tx.commit().await?;

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
