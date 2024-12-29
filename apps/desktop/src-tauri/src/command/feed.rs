use colette_api::{
    feed::{Feed, FeedCreate, FeedDetect, FeedDetected, FeedListQuery, FeedUpdate},
    Paginated, Session,
};
use colette_core::feed::FeedService;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn list_feeds(
    service: State<'_, FeedService>,
    session: State<'_, Session>,
    query: FeedListQuery,
) -> Result<Paginated<Feed>, String> {
    let feeds = service
        .list_feeds(query.into(), session.user_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(feeds.into())
}

#[tauri::command]
pub async fn create_feed(
    service: State<'_, FeedService>,
    session: State<'_, Session>,
    data: FeedCreate,
) -> Result<Feed, String> {
    let feed = service
        .create_feed(data.into(), session.user_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(feed.into())
}

#[tauri::command]
pub async fn get_feed(
    service: State<'_, FeedService>,
    session: State<'_, Session>,
    id: Uuid,
) -> Result<Feed, String> {
    let feed = service
        .get_feed(id, session.user_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(feed.into())
}

#[tauri::command]
pub async fn update_feed(
    service: State<'_, FeedService>,
    session: State<'_, Session>,
    id: Uuid,
    data: FeedUpdate,
) -> Result<Feed, String> {
    let feed = service
        .update_feed(id, data.into(), session.user_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(feed.into())
}

#[tauri::command]
pub async fn delete_feed(
    service: State<'_, FeedService>,
    session: State<'_, Session>,
    id: Uuid,
) -> Result<(), String> {
    service
        .delete_feed(id, session.user_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn detect_feeds(
    service: State<'_, FeedService>,
    data: FeedDetect,
) -> Result<Paginated<FeedDetected>, String> {
    let feeds = service
        .detect_feeds(data.into())
        .await
        .map_err(|e| e.to_string())?;

    Ok(feeds.into())
}
