pub use backup::PostgresBackupRepository;
pub use bookmark::PostgresBookmarkRepository;
use colette_core::filter::{BooleanOp, DateOp, NumberOp, TextOp};
pub use collection::PostgresCollectionRepository;
pub use feed::PostgresFeedRepository;
pub use feed_entry::PostgresFeedEntryRepository;
pub use pat::PostgresPatRepository;
use sqlx::{
    Decode, Encode, Postgres, Type,
    encode::IsNull,
    error::BoxDynError,
    postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef},
};
pub use subscription::PostgresSubscriptionRepository;
pub use subscription_entry::PostgresSubscriptionEntryRepository;
pub use tag::PostgresTagRepository;
use url::Url;
pub use user::PostgresUserRepository;

mod backup;
mod bookmark;
mod collection;
mod feed;
mod feed_entry;
mod pat;
mod subscription;
mod subscription_entry;
mod tag;
mod user;

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct DbUrl(Url);

impl From<DbUrl> for Url {
    fn from(value: DbUrl) -> Self {
        value.0
    }
}

impl From<Url> for DbUrl {
    fn from(value: Url) -> Self {
        DbUrl(value)
    }
}

impl Type<Postgres> for DbUrl {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("text")
    }
}

impl PgHasArrayType for DbUrl {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::array_of("text")
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

pub(crate) trait ToColumn {
    fn to_column(self) -> String;
}

pub(crate) trait ToSql {
    fn to_sql(self) -> String;
}

impl ToSql for (&str, TextOp) {
    fn to_sql(self) -> String {
        let (column, op) = self;

        match op {
            TextOp::Equals(value) => format!("{column} = {value}"),
            TextOp::Contains(value) => format!("{column} LIKE %{value}%"),
            TextOp::StartsWith(value) => format!("{column} LIKE {value}%"),
            TextOp::EndsWith(value) => format!("{column} LIKE %{value}"),
        }
    }
}

impl ToSql for (&str, NumberOp) {
    fn to_sql(self) -> String {
        let (column, op) = self;

        match op {
            NumberOp::Equals(value) => format!("{column} = {value}"),
            NumberOp::LessThan(value) => format!("{column} < {value}"),
            NumberOp::GreaterThan(value) => format!("{column} > {value}"),
            NumberOp::Between(value) => {
                format!("{} BETWEEN {} AND {}", column, value.start, value.end)
            }
        }
    }
}

impl ToSql for (&str, BooleanOp) {
    fn to_sql(self) -> String {
        let (column, op) = self;

        match op {
            BooleanOp::Equals(value) => format!("{column} = {value}"),
        }
    }
}

impl ToSql for (&str, DateOp) {
    fn to_sql(self) -> String {
        let (column, op) = self;

        match op {
            DateOp::Before(value) => format!("{column} < {value}"),
            DateOp::After(value) => format!("{column} > {value}"),
            DateOp::Between(value) => {
                format!("{} BETWEEN {} AND {}", column, value.start, value.end)
            }
            DateOp::InLast(value) => {
                format!("(extract(epoch FROM now()) - {column}) < {value}")
            }
        }
    }
}
