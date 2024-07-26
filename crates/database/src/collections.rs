use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SelectDefaultParams {
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct UpdateParams<'a> {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub title: Option<&'a str>,
}
