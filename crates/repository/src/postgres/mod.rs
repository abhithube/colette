pub use account::PostgresAccountRepository;
pub use api_key::PostgresApiKeyRepository;
pub use backup::PostgresBackupRepository;
pub use bookmark::PostgresBookmarkRepository;
pub use collection::PostgresCollectionRepository;
pub use feed::PostgresFeedRepository;
pub use feed_entry::PostgresFeedEntryRepository;
pub use job::PostgresJobRepository;
use sqlx::{
    Decode, Encode, Postgres, Type,
    encode::IsNull,
    error::BoxDynError,
    postgres::{PgArgumentBuffer, PgTypeInfo, PgValueFormat, PgValueRef},
};
pub use subscription::PostgresSubscriptionRepository;
pub use subscription_entry::PostgresSubscriptionEntryRepository;
pub use tag::PostgresTagRepository;
use url::Url;
pub use user::PostgresUserRepository;
use uuid::Uuid;

mod account;
mod api_key;
mod backup;
mod bookmark;
mod collection;
mod feed;
mod feed_entry;
mod job;
mod subscription;
mod subscription_entry;
mod tag;
mod user;

#[derive(Debug)]
pub(crate) struct DbUrl(Url);

impl Type<Postgres> for DbUrl {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("text")
    }
}

impl Encode<'_, Postgres> for DbUrl {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.extend_from_slice(self.0.as_str().as_bytes());

        Ok(IsNull::No)
    }
}

impl Decode<'_, Postgres> for DbUrl {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.format() {
            PgValueFormat::Binary => Url::parse(str::from_utf8(value.as_bytes()?)?),
            PgValueFormat::Text => Url::parse(value.as_str()?),
        }
        .map(DbUrl)
        .map_err(Into::into)
    }
}

#[derive(Debug, sqlx::FromRow)]
struct IdRow {
    id: Uuid,
}
