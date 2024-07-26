use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SelectParams<'a> {
    pub bookmark_id: &'a Uuid,
    pub tag_id: &'a Uuid,
}
