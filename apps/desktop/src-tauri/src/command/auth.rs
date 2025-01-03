use colette_api::{
    auth::{Login, Register, User},
    Session,
};
use colette_core::auth::AuthService;
use tauri::State;

#[tauri::command]
pub async fn register(service: State<'_, AuthService>, data: Register) -> Result<User, String> {
    let user = service
        .register(data.into())
        .await
        .map_err(|e| e.to_string())?;

    Ok(user.into())
}

#[tauri::command]
pub async fn login(service: State<'_, AuthService>, data: Login) -> Result<User, String> {
    let user = service
        .login(data.into())
        .await
        .map_err(|e| e.to_string())?;

    Ok(user.into())
}

#[tauri::command]
pub async fn get_active_user(
    service: State<'_, AuthService>,
    session: State<'_, Session>,
) -> Result<User, String> {
    let user = service
        .get_active(session.user_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(user.into())
}
