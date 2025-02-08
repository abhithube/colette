use chrono::{DateTime, Utc};
use colette_core::{
    Bookmark, Feed, Tag, bookmark,
    feed::{self, ProcessedFeed},
    folder,
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

struct BookmarkRow {
    id: Uuid,
    link: DbUrl,
    title: String,
    thumbnail_url: Option<DbUrl>,
    published_at: Option<DateTime<Utc>>,
    author: Option<String>,
    archived_path: Option<String>,
    folder_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    tags: Option<Json<Vec<Tag>>>,
}

impl From<BookmarkRow> for Bookmark {
    fn from(value: BookmarkRow) -> Self {
        Self {
            id: value.id,
            link: value.link.0,
            title: value.title,
            thumbnail_url: value.thumbnail_url.map(|e| e.0),
            published_at: value.published_at,
            author: value.author,
            archived_path: value.archived_path,
            folder_id: value.folder_id,
            created_at: value.created_at,
            tags: value.tags.map(|e| e.0),
        }
    }
}

pub(crate) async fn select_bookmarks<'a>(
    ex: impl PgExecutor<'a>,
    id: Option<Uuid>,
    folder_id: Option<Option<Uuid>>,
    user_id: Uuid,
    cursor: Option<bookmark::Cursor>,
    limit: Option<i64>,
    tags: Option<Vec<String>>,
) -> sqlx::Result<Vec<Bookmark>> {
    let (has_folder, folder_id) = match folder_id {
        Some(folder_id) => (true, folder_id),
        None => (false, None),
    };

    let (has_cursor, cursor_created_at) = cursor
        .map(|e| (true, Some(e.created_at)))
        .unwrap_or_default();

    sqlx::query_file_as!(
        BookmarkRow,
        "queries/bookmarks/select.sql",
        user_id,
        id.is_none(),
        id,
        !has_folder,
        folder_id,
        tags.is_none(),
        &tags
            .unwrap_or_default()
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>(),
        !has_cursor,
        cursor_created_at,
        limit
    )
    .fetch_all(ex)
    .await
    .map(|e| e.into_iter().map(Bookmark::from).collect())
}

struct FeedRow {
    id: Uuid,
    link: DbUrl,
    title: String,
    xml_url: Option<DbUrl>,
    folder_id: Option<Uuid>,
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
            folder_id: value.folder_id,
            tags: value.tags.map(|e| e.0),
            unread_count: value.unread_count,
        }
    }
}

pub(crate) async fn select_feeds<'a>(
    ex: impl PgExecutor<'a>,
    id: Option<Uuid>,
    folder_id: Option<Option<Uuid>>,
    user_id: Uuid,
    cursor: Option<feed::Cursor>,
    limit: Option<i64>,
    tags: Option<Vec<String>>,
) -> sqlx::Result<Vec<Feed>> {
    let (has_folder, folder_id) = match folder_id {
        Some(folder_id) => (true, folder_id),
        None => (false, None),
    };

    let (has_cursor, cursor_title, cursor_id) = cursor
        .map(|e| (true, Some(e.title), Some(e.id)))
        .unwrap_or_default();

    sqlx::query_file_as!(
        FeedRow,
        "queries/user_feeds/select.sql",
        user_id,
        id.is_none(),
        id,
        !has_folder,
        folder_id,
        tags.is_none(),
        &tags.unwrap_or_default(),
        !has_cursor,
        cursor_title,
        cursor_id,
        limit
    )
    .fetch_all(ex)
    .await
    .map(|e| e.into_iter().map(Feed::from).collect())
}

pub async fn select_folders<'a>(
    ex: impl PgExecutor<'a>,
    id: Option<Uuid>,
    user_id: Uuid,
    parent_id: Option<Option<Uuid>>,
    limit: Option<i64>,
    cursor: Option<folder::Cursor>,
) -> sqlx::Result<Vec<colette_core::Folder>> {
    let (has_parent, parent_id) = match parent_id {
        Some(parent_id) => (true, parent_id),
        None => (false, None),
    };

    let (has_cursor, cursor_title) = cursor.map(|e| (true, Some(e.title))).unwrap_or_default();

    sqlx::query_file_as!(
        colette_core::Folder,
        "queries/folders/select.sql",
        user_id,
        id.is_none(),
        id,
        !has_parent,
        parent_id,
        !has_cursor,
        cursor_title,
        limit
    )
    .fetch_all(ex)
    .await
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
        thumbnail_urls.push(item.thumbnail.map(String::from));
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
