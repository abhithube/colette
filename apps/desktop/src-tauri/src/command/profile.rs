use std::sync::Arc;

use colette_api::{
    profile::{Profile, ProfileCreate, ProfileUpdate},
    Paginated, Session,
};
use colette_core::profile::ProfileService;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn list_profiles(
    service: State<'_, Arc<ProfileService>>,
    session: State<'_, Session>,
) -> Result<Paginated<Profile>, String> {
    let profiles = service
        .list_profiles(session.user_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(profiles.into())
}

#[tauri::command]
pub async fn create_profile(
    service: State<'_, Arc<ProfileService>>,
    session: State<'_, Session>,
    data: ProfileCreate,
) -> Result<Profile, String> {
    let profile = service
        .create_profile(data.into(), session.user_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(profile.into())
}

#[tauri::command]
pub async fn get_profile(
    service: State<'_, Arc<ProfileService>>,
    session: State<'_, Session>,
    id: Uuid,
) -> Result<Profile, String> {
    let profile = service
        .get_profile(id, session.user_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(profile.into())
}

#[tauri::command]
pub async fn get_active_profile(
    service: State<'_, Arc<ProfileService>>,
    session: State<'_, Session>,
) -> Result<Profile, String> {
    let profile = service
        .get_profile(session.profile_id, session.user_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(profile.into())
}

#[tauri::command]
pub async fn update_profile(
    service: State<'_, Arc<ProfileService>>,
    session: State<'_, Session>,
    id: Uuid,
    data: ProfileUpdate,
) -> Result<Profile, String> {
    let profile = service
        .update_profile(id, data.into(), session.user_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(profile.into())
}

#[tauri::command]
pub async fn delete_profile(
    service: State<'_, Arc<ProfileService>>,
    session: State<'_, Session>,
    id: Uuid,
) -> Result<(), String> {
    service
        .delete_profile(id, session.user_id)
        .await
        .map_err(|e| e.to_string())
}
