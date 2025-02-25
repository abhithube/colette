use chrono::{DateTime, Utc};
use colette_core::{
    Feed, Tag,
    feed::{self, ProcessedFeed},
    filter::{BooleanOp, DateOp, NumberOp, TextOp},
};
use sqlx::{
    Database, Decode, Encode, PgExecutor, Postgres, Type,
    encode::IsNull,
    error::BoxDynError,
    postgres::{PgTypeInfo, PgValueRef},
    types::Json,
};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DbUrl(pub Url);

impl Type<Postgres> for DbUrl {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("TEXT")
    }
}

impl Encode<'_, Postgres> for DbUrl {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<'_, Postgres>>::encode_by_ref(&self.0.as_str(), buf)
    }
}

impl Decode<'_, Postgres> for DbUrl {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        let url_str = <String as Decode<Postgres>>::decode(value)?;
        Ok(DbUrl(Url::parse(&url_str)?))
    }
}

struct FeedRow {
    id: Uuid,
    link: DbUrl,
    title: String,
    xml_url: Option<DbUrl>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
    tags: Option<Json<Vec<Tag>>>,
    unread_count: Option<i64>,
}

impl From<FeedRow> for Feed {
    fn from(value: FeedRow) -> Self {
        Self {
            id: value.id,
            link: value.link.0,
            title: value.title,
            xml_url: value.xml_url.map(|e| e.0),
            created_at: value.created_at,
            updated_at: value.updated_at,
            tags: value.tags.map(|e| e.0),
            unread_count: value.unread_count,
        }
    }
}

pub(crate) async fn select_feeds<'a>(
    ex: impl PgExecutor<'a>,
    id: Option<Uuid>,
    user_id: Uuid,
    cursor: Option<feed::Cursor>,
    limit: Option<i64>,
    tags: Option<Vec<String>>,
) -> sqlx::Result<Vec<Feed>> {
    let cursor_id = cursor.as_ref().map(|e| e.id);

    sqlx::query_file_as!(
        FeedRow,
        "queries/user_feeds/select.sql",
        user_id,
        id.is_none(),
        id,
        tags.is_none(),
        &tags.unwrap_or_default(),
        cursor.is_none(),
        cursor.map(|e| e.title),
        cursor_id,
        limit
    )
    .fetch_all(ex)
    .await
    .map(|e| e.into_iter().map(Into::into).collect())
}

pub(crate) async fn insert_feed_with_entries<'a>(
    ex: impl PgExecutor<'a>,
    url: Url,
    feed: ProcessedFeed,
) -> sqlx::Result<Uuid> {
    let mut links = Vec::<String>::new();
    let mut titles = Vec::<String>::new();
    let mut published_ats = Vec::<DateTime<Utc>>::new();
    let mut descriptions = Vec::<Option<String>>::new();
    let mut authors = Vec::<Option<String>>::new();
    let mut thumbnail_urls = Vec::<Option<String>>::new();

    for item in feed.entries {
        links.push(item.link.to_string());
        titles.push(item.title);
        published_ats.push(item.published);
        descriptions.push(item.description);
        authors.push(item.author);
        thumbnail_urls.push(item.thumbnail.map(Into::into));
    }

    let feed_id = {
        let link = feed.link;
        let xml_url = if url == link { None } else { Some(DbUrl(url)) };

        sqlx::query_file_scalar!(
            "queries/feeds/insert_with_entries.sql",
            DbUrl(link) as DbUrl,
            xml_url as Option<DbUrl>,
            &links,
            &titles,
            &published_ats,
            &descriptions as &[Option<String>],
            &authors as &[Option<String>],
            &thumbnail_urls as &[Option<String>],
        )
        .fetch_one(ex)
        .await?
    };

    Ok(feed_id)
}

pub(crate) trait ToColumn {
    fn to_column(&self) -> String;
}

pub(crate) trait ToSql {
    fn to_sql(&self) -> String;
}

impl ToSql for (String, &TextOp) {
    fn to_sql(&self) -> String {
        let (column, op) = self;

        match op {
            TextOp::Equals(value) => format!("{} = '{}'", column, value),
            TextOp::Contains(value) => format!("{} ILIKE '%{}%'", column, value),
            TextOp::StartsWith(value) => format!("{} ILIKE '{}%'", column, value),
            TextOp::EndsWith(value) => format!("{} ILIKE '%{}'", column, value),
        }
    }
}

impl ToSql for (String, &NumberOp) {
    fn to_sql(&self) -> String {
        let (column, op) = self;

        match op {
            NumberOp::Equals(value) => format!("{} = {}", column, value),
            NumberOp::LessThan(value) => format!("{} < {}", column, value),
            NumberOp::GreaterThan(value) => format!("{} > {}", column, value),
            NumberOp::Between(value) => format!(
                "{} > {} AND {} < {}",
                column, value.start, column, value.end
            ),
        }
    }
}

impl ToSql for (String, &BooleanOp) {
    fn to_sql(&self) -> String {
        let (column, op) = self;

        match op {
            BooleanOp::Equals(value) => format!("{} = {}", column, value),
        }
    }
}

impl ToSql for (String, &DateOp) {
    fn to_sql(&self) -> String {
        let (column, op) = self;

        match op {
            DateOp::Before(value) => format!("{} < '{}'", column, value),
            DateOp::After(value) => format!("{} > '{}'", column, value),
            DateOp::Between(value) => format!(
                "{} > '{}' AND {} < '{}'",
                column, value.start, column, value.end
            ),
            DateOp::InLast(value) => format!(
                "round((extract(epoch FROM now()) - extract(epoch FROM {})) * 1000) < '{}'",
                column, value
            ),
        }
    }
}
