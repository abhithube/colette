use colette_api::{
    tag::{Tag, TagCreate, TagListQuery, TagUpdate},
    Paginated, Session,
};
use colette_core::tag::TagService;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn list_tags(
    service: State<'_, TagService>,
    session: State<'_, Session>,
    query: TagListQuery,
) -> Result<Paginated<Tag>, String> {
    let tags = service
        .list_tags(query.into(), session.user_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(tags.into())
}

#[tauri::command]
pub async fn create_tag(
    service: State<'_, TagService>,
    session: State<'_, Session>,
    data: TagCreate,
) -> Result<Tag, String> {
    let tag = service
        .create_tag(data.into(), session.user_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(tag.into())
}

#[tauri::command]
pub async fn get_tag(
    service: State<'_, TagService>,
    session: State<'_, Session>,
    id: Uuid,
) -> Result<Tag, String> {
    let tag = service
        .get_tag(id, session.user_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(tag.into())
}

#[tauri::command]
pub async fn update_tag(
    service: State<'_, TagService>,
    session: State<'_, Session>,
    id: Uuid,
    data: TagUpdate,
) -> Result<Tag, String> {
    let tag = service
        .update_tag(id, data.into(), session.user_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(tag.into())
}

#[tauri::command]
pub async fn delete_tag(
    service: State<'_, TagService>,
    session: State<'_, Session>,
    id: Uuid,
) -> Result<(), String> {
    service
        .delete_tag(id, session.user_id)
        .await
        .map_err(|e| e.to_string())
}
