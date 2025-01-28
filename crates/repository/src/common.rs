use chrono::{DateTime, Utc};
use colette_core::{bookmark, feed, folder, Bookmark, Feed, Tag};
use sqlx::{types::Json, PgExecutor};
use uuid::Uuid;

pub struct BookmarkRow {
    pub id: Uuid,
    pub link: String,
    pub title: Option<String>,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub original_title: String,
    pub original_thumbnail_url: Option<String>,
    pub original_published_at: Option<DateTime<Utc>>,
    pub original_author: Option<String>,
    pub folder_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub tags: Option<Json<Vec<Tag>>>,
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
            original_title: value.original_title,
            original_thumbnail_url: value.original_thumbnail_url,
            original_published_at: value.original_published_at,
            original_author: value.original_author,
            folder_id: value.folder_id,
            created_at: value.created_at,
            tags: value.tags.map(|e| e.0),
        }
    }
}

pub async fn select_bookmarks<'a>(
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
        "queries/user_bookmarks/select.sql",
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

pub struct FeedRow {
    pub id: Uuid,
    pub link: String,
    pub title: Option<String>,
    pub xml_url: Option<String>,
    pub original_title: String,
    pub folder_id: Option<Uuid>,
    pub tags: Option<Json<Vec<Tag>>>,
    pub unread_count: Option<i64>,
}

impl From<FeedRow> for Feed {
    fn from(value: FeedRow) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            xml_url: value.xml_url,
            original_title: value.original_title,
            folder_id: value.folder_id,
            tags: value.tags.map(|e| e.0),
            unread_count: value.unread_count,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn select_feeds<'a>(
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
