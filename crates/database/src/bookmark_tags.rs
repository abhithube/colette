use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SelectParams {
    pub bookmark_id: Uuid,
    pub tag_id: Uuid,
}
