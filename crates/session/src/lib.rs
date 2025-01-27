use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use redis::{AsyncCommands, ExistenceCheck, SetExpiry, SetOptions};
use tokio::sync::Mutex;
use tower_sessions_core::{
    session::{Id, Record},
    session_store, SessionStore,
};

#[derive(Debug, Clone, Default)]
pub struct RedisStore<C: AsyncCommands + Send + Sync> {
    client: Arc<Mutex<C>>,
}

impl<C: AsyncCommands + Send + Sync> RedisStore<C> {
    pub fn new(client: C) -> Self {
        Self {
            client: Arc::new(Mutex::new(client)),
        }
    }

    async fn save_with_options(
        &self,
        record: &Record,
        check: Option<ExistenceCheck>,
    ) -> session_store::Result<bool> {
        let mut client = self.client.lock().await;

        let options = SetOptions::default()
            .get(false)
            .with_expiration(SetExpiry::EXAT(record.expiry_date.unix_timestamp() as u64));

        if let Some(check) = check {
            options.conditional_set(check);
        }

        client
            .set_options(
                record.id.to_string(),
                serde_json::to_vec(&record)
                    .map_err(|e| session_store::Error::Encode(e.to_string()))?
                    .as_slice(),
                options,
            )
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))
    }
}

#[async_trait]
impl<C> SessionStore for RedisStore<C>
where
    C: AsyncCommands + Send + Sync + Debug + 'static,
{
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        loop {
            if !self
                .save_with_options(record, Some(ExistenceCheck::NX))
                .await?
            {
                record.id = Id::default();
                continue;
            }
            break;
        }

        Ok(())
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        self.save_with_options(record, Some(ExistenceCheck::XX))
            .await?;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let mut client = self.client.lock().await;

        let data = client
            .get::<_, Option<Vec<u8>>>(session_id.to_string())
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        if let Some(data) = data {
            Ok(Some(serde_json::from_slice(&data).map_err(|e| {
                session_store::Error::Decode(e.to_string())
            })?))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        let mut client = self.client.lock().await;

        client
            .del::<_, ()>(session_id.to_string())
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }
}
