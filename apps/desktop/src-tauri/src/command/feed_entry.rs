use colette_api::{
    feed_entry::{FeedEntry, FeedEntryListQuery, FeedEntryUpdate},
    Paginated, Session,
};
use colette_core::feed_entry::FeedEntryService;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn list_feed_entries(
    service: State<'_, FeedEntryService>,
    session: Session,
    query: FeedEntryListQuery,
) -> Result<Paginated<FeedEntry>, String> {
    let entries = service
        .list_feed_entries(query.into(), session.profile_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(entries.into())
}

#[tauri::command]
pub async fn get_feed_entry(
    service: State<'_, FeedEntryService>,
    session: State<'_, Session>,
    id: Uuid,
) -> Result<FeedEntry, String> {
    let entry = service
        .get_feed_entry(id, session.profile_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(entry.into())
}

#[tauri::command]
pub async fn update_feed_entry(
    service: State<'_, FeedEntryService>,
    session: State<'_, Session>,
    id: Uuid,
    data: FeedEntryUpdate,
) -> Result<FeedEntry, String> {
    let entry = service
        .update_feed_entry(id, data.into(), session.profile_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(entry.into())
}
