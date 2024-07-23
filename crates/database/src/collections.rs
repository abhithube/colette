use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SelectDefaultParams<'a> {
    pub profile_id: &'a Uuid,
}

#[derive(Clone, Debug)]
pub struct UpdateParams<'a> {
    pub id: &'a Uuid,
    pub profile_id: &'a Uuid,
    pub title: Option<&'a str>,
}
