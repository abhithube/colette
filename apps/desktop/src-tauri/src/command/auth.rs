use std::sync::Arc;

use colette_api::{
    auth::{Login, Register, SwitchProfile, User},
    profile::Profile,
    Session,
};
use colette_core::auth::AuthService;
use tauri::State;

#[tauri::command]
pub async fn register(
    service: State<'_, Arc<AuthService>>,
    data: Register,
) -> Result<User, String> {
    let user = service
        .register(data.into())
        .await
        .map_err(|e| e.to_string())?;

    Ok(user.into())
}

#[tauri::command]
pub async fn login(service: State<'_, Arc<AuthService>>, data: Login) -> Result<Profile, String> {
    let profile = service
        .login(data.into())
        .await
        .map_err(|e| e.to_string())?;

    Ok(profile.into())
}

#[tauri::command]
pub async fn get_active_user(
    service: State<'_, Arc<AuthService>>,
    session: State<'_, Session>,
) -> Result<User, String> {
    let user = service
        .get_active(session.user_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(user.into())
}

#[tauri::command]
pub async fn switch_profile(
    service: State<'_, Arc<AuthService>>,
    session: State<'_, Session>,
    data: SwitchProfile,
) -> Result<Profile, String> {
    let profile = service
        .switch_profile(data.into(), session.user_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(profile.into())
}
