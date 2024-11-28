pub use store::KvSessionStore;
use worker::kv::{KvError, KvStore};

mod store;

#[worker::send]
pub(crate) async fn get(kv: &KvStore, key: &str) -> Result<Option<Vec<u8>>, KvError> {
    kv.get(key).bytes().await
}

#[worker::send]
pub(crate) async fn put(
    kv: &KvStore,
    key: &str,
    value: &[u8],
    expiration: u64,
) -> Result<(), KvError> {
    kv.put_bytes(key, value)
        .unwrap()
        .expiration(expiration)
        .execute()
        .await
}

#[worker::send]
pub(crate) async fn delete(kv: &KvStore, key: &str) -> Result<(), KvError> {
    kv.delete(key).await
}
