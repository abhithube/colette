use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SelectParams<'a> {
    pub profile_feed_id: &'a Uuid,
    pub tag_id: &'a Uuid,
}
