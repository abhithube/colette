use colette_api::Session;
use colette_core::backup::BackupService;
use tauri::State;

#[tauri::command]
pub async fn import_opml(
    service: State<'_, BackupService>,
    session: State<'_, Session>,
    data: Vec<u8>,
) -> Result<(), String> {
    service
        .import_opml(data.into(), session.profile_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_opml(
    service: State<'_, BackupService>,
    session: State<'_, Session>,
) -> Result<Vec<u8>, String> {
    let bytes = service
        .export_opml(session.profile_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(bytes.into())
}

#[tauri::command]
pub async fn import_netscape(
    service: State<'_, BackupService>,
    session: State<'_, Session>,
    data: Vec<u8>,
) -> Result<(), String> {
    service
        .import_netscape(data.into(), session.profile_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_netscape(
    service: State<'_, BackupService>,
    session: State<'_, Session>,
) -> Result<Vec<u8>, String> {
    let bytes = service
        .export_netscape(session.profile_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(bytes.into())
}
