use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct UpdateParams<'a> {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub custom_title: Option<&'a str>,
}
