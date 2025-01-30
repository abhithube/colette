use chrono::{DateTime, Utc};
use colette_core::{
    bookmark,
    feed::{self, ProcessedFeed},
    folder, Bookmark, Feed, Tag,
};
use sqlx::{types::Json, PgExecutor};
use uuid::Uuid;

struct BookmarkRow {
    id: Uuid,
    link: String,
    title: String,
    thumbnail_url: Option<String>,
    published_at: Option<DateTime<Utc>>,
    author: Option<String>,
    folder_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    tags: Option<Json<Vec<Tag>>>,
}

impl From<BookmarkRow> for Bookmark {
    fn from(value: BookmarkRow) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author,
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
        &tags.unwrap_or_default(),
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
    link: String,
    title: String,
    xml_url: Option<String>,
    folder_id: Option<Uuid>,
    tags: Option<Json<Vec<Tag>>>,
    unread_count: Option<i64>,
}

impl From<FeedRow> for Feed {
    fn from(value: FeedRow) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            xml_url: value.xml_url,
            folder_id: value.folder_id,
            tags: value.tags.map(|e| e.0),
            unread_count: value.unread_count,
        }
    }
}

#[allow(clippy::too_many_arguments)]
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
    url: String,
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
        let link = feed.link.to_string();
        let xml_url = if url == link { None } else { Some(url) };

        sqlx::query_file_scalar!(
            "queries/feeds/insert_with_entries.sql",
            link,
            xml_url,
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
