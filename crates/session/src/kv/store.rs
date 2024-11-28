use std::fmt;

use tower_sessions_core::{
    session::{Id, Record},
    session_store, SessionStore,
};
use worker::kv::KvStore;

#[derive(Clone)]
pub struct KvSessionStore {
    kv: KvStore,
}

impl fmt::Debug for KvSessionStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

impl KvSessionStore {
    pub fn new(kv: KvStore) -> Self {
        Self { kv }
    }
}

#[async_trait::async_trait]
impl SessionStore for KvSessionStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        let payload =
            serde_json::to_vec(record).map_err(|e| session_store::Error::Encode(e.to_string()))?;

        loop {
            let key = &record.id.to_string();

            let session = super::get(&self.kv, key)
                .await
                .map_err(|e| session_store::Error::Backend(e.to_string()))?;

            if session.is_none() {
                super::put(
                    &self.kv,
                    key,
                    &payload,
                    record.expiry_date.unix_timestamp() as u64,
                )
                .await
                .map_err(|e| session_store::Error::Backend(e.to_string()))?;

                break;
            }

            record.id = Id::default();
        }

        Ok(())
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let payload =
            serde_json::to_vec(record).map_err(|e| session_store::Error::Encode(e.to_string()))?;

        super::put(
            &self.kv,
            &record.id.to_string(),
            &payload,
            record.expiry_date.unix_timestamp() as u64,
        )
        .await
        .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let data = super::get(&self.kv, &session_id.to_string())
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
        super::delete(&self.kv, &session_id.to_string())
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))
    }
}
