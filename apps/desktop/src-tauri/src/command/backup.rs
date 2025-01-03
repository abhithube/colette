use colette_api::Session;
use colette_core::backup::BackupService;
use colette_queue::Queue;
use colette_task::{import_bookmarks, import_feeds};
use tauri::State;

#[tauri::command]
pub async fn import_opml(
    service: State<'_, BackupService>,
    import_feeds_queue: State<'_, Box<dyn Queue<Data = import_feeds::Data>>>,
    session: State<'_, Session>,
    data: Vec<u8>,
) -> Result<(), String> {
    let urls = service
        .import_opml(data.into(), session.user_id)
        .await
        .map_err(|e| e.to_string())?;

    import_feeds_queue
        .push(import_feeds::Data { urls })
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn export_opml(
    service: State<'_, BackupService>,
    session: State<'_, Session>,
) -> Result<Vec<u8>, String> {
    let bytes = service
        .export_opml(session.user_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(bytes.into())
}

#[tauri::command]
pub async fn import_netscape(
    service: State<'_, BackupService>,
    import_bookmarks_queue: State<'_, Box<dyn Queue<Data = import_bookmarks::Data>>>,
    session: State<'_, Session>,
    data: Vec<u8>,
) -> Result<(), String> {
    let urls = service
        .import_netscape(data.into(), session.user_id)
        .await
        .map_err(|e| e.to_string())?;

    import_bookmarks_queue
        .push(import_bookmarks::Data { urls })
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn export_netscape(
    service: State<'_, BackupService>,
    session: State<'_, Session>,
) -> Result<Vec<u8>, String> {
    let bytes = service
        .export_netscape(session.user_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(bytes.into())
}
