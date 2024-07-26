use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SelectParams {
    pub profile_feed_id: Uuid,
    pub tag_id: Uuid,
}
