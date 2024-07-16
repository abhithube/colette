use colette_core::profiles::{ProfileFindByIdParams, ProfileFindManyParams};
use uuid::Uuid;

#[derive(Debug)]
pub struct SelectManyParams<'a> {
    pub user_id: &'a Uuid,
}

impl<'a> From<&'a ProfileFindManyParams> for SelectManyParams<'a> {
    fn from(value: &'a ProfileFindManyParams) -> Self {
        Self {
            user_id: &value.user_id,
        }
    }
}

#[derive(Debug)]
pub struct SelectByIdParams<'a> {
    pub id: &'a Uuid,
    pub user_id: &'a Uuid,
}

impl<'a> From<&'a ProfileFindByIdParams> for SelectByIdParams<'a> {
    fn from(value: &'a ProfileFindByIdParams) -> Self {
        Self {
            id: &value.id,
            user_id: &value.user_id,
        }
    }
}

#[derive(Debug)]
pub struct SelectDefaultParams<'a> {
    pub user_id: &'a Uuid,
}

#[derive(Debug)]
pub struct UpdateParams<'a> {
    pub id: &'a Uuid,
    pub user_id: &'a Uuid,
    pub title: Option<&'a str>,
    pub image_url: Option<&'a str>,
}
