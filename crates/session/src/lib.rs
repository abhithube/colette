use ::redis::aio::MultiplexedConnection;
pub use redis::RedisStore;
use tower_sessions_core::{
    SessionStore,
    session::{Id, Record},
    session_store,
};
use tower_sessions_sqlx_store::SqliteStore;

mod redis;

#[derive(Debug, Clone)]
pub enum SessionAdapter {
    Redis(RedisStore<MultiplexedConnection>),
    Sqlite(SqliteStore),
}

#[async_trait::async_trait]
impl SessionStore for SessionAdapter {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        match self {
            Self::Redis(store) => store.create(record).await,
            Self::Sqlite(store) => store.create(record).await,
        }
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        match self {
            Self::Redis(store) => store.save(record).await,
            Self::Sqlite(store) => store.save(record).await,
        }
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        match self {
            Self::Redis(store) => store.load(session_id).await,
            Self::Sqlite(store) => store.load(session_id).await,
        }
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        match self {
            Self::Redis(store) => store.delete(session_id).await,
            Self::Sqlite(store) => store.delete(session_id).await,
        }
    }
}
