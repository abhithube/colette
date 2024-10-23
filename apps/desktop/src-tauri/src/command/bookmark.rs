use std::sync::Arc;

use colette_api::{
    bookmark::{Bookmark, BookmarkCreate, BookmarkListQuery, BookmarkUpdate},
    Paginated, Session,
};
use colette_core::bookmark::BookmarkService;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn list_bookmarks(
    service: State<'_, Arc<BookmarkService>>,
    session: State<'_, Session>,
    query: BookmarkListQuery,
) -> Result<Paginated<Bookmark>, String> {
    let bookmarks = service
        .list_bookmarks(query.into(), session.profile_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(bookmarks.into())
}

#[tauri::command]
pub async fn create_bookmark(
    service: State<'_, Arc<BookmarkService>>,
    session: State<'_, Session>,
    data: BookmarkCreate,
) -> Result<Bookmark, String> {
    let bookmark = service
        .create_bookmark(data.into(), session.profile_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(bookmark.into())
}

#[tauri::command]
pub async fn get_bookmark(
    service: State<'_, Arc<BookmarkService>>,
    session: State<'_, Session>,
    id: Uuid,
) -> Result<Bookmark, String> {
    let bookmark = service
        .get_bookmark(id, session.profile_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(bookmark.into())
}

#[tauri::command]
pub async fn update_bookmark(
    service: State<'_, Arc<BookmarkService>>,
    session: State<'_, Session>,
    id: Uuid,
    data: BookmarkUpdate,
) -> Result<Bookmark, String> {
    let bookmark = service
        .update_bookmark(id, data.into(), session.profile_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(bookmark.into())
}

#[tauri::command]
pub async fn delete_bookmark(
    service: State<'_, Arc<BookmarkService>>,
    session: State<'_, Session>,
    id: Uuid,
) -> Result<(), String> {
    service
        .delete_bookmark(id, session.profile_id)
        .await
        .map_err(|e| e.to_string())
}
