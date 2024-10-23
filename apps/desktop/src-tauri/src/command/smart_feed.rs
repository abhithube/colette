use colette_api::{
    smart_feed::{SmartFeed, SmartFeedCreate, SmartFeedUpdate},
    Paginated, Session,
};
use colette_core::smart_feed::SmartFeedService;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn list_smart_feeds(
    service: State<'_, SmartFeedService>,
    session: State<'_, Session>,
) -> Result<Paginated<SmartFeed>, String> {
    let smart_feeds = service
        .list_smart_feeds(session.profile_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(smart_feeds.into())
}

#[tauri::command]
pub async fn create_smart_feed(
    service: State<'_, SmartFeedService>,
    session: State<'_, Session>,
    data: SmartFeedCreate,
) -> Result<SmartFeed, String> {
    let smart_feed = service
        .create_smart_feed(data.into(), session.profile_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(smart_feed.into())
}

#[tauri::command]
pub async fn get_smart_feed(
    service: State<'_, SmartFeedService>,
    session: State<'_, Session>,
    id: Uuid,
) -> Result<SmartFeed, String> {
    let smart_feed = service
        .get_smart_feed(id, session.profile_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(smart_feed.into())
}

#[tauri::command]
pub async fn update_smart_feed(
    service: State<'_, SmartFeedService>,
    session: State<'_, Session>,
    id: Uuid,
    data: SmartFeedUpdate,
) -> Result<SmartFeed, String> {
    let smart_feed = service
        .update_smart_feed(id, data.into(), session.profile_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(smart_feed.into())
}

#[tauri::command]
pub async fn delete_smart_feed(
    service: State<'_, SmartFeedService>,
    session: State<'_, Session>,
    id: Uuid,
) -> Result<(), String> {
    service
        .delete_smart_feed(id, session.profile_id)
        .await
        .map_err(|e| e.to_string())
}
