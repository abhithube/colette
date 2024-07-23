use colette_core::profiles::{ProfilesFindByIdParams, ProfilesFindManyParams};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SelectManyParams<'a> {
    pub user_id: &'a Uuid,
}

impl<'a> From<&'a ProfilesFindManyParams> for SelectManyParams<'a> {
    fn from(value: &'a ProfilesFindManyParams) -> Self {
        Self {
            user_id: &value.user_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SelectByIdParams<'a> {
    pub id: &'a Uuid,
    pub user_id: &'a Uuid,
}

impl<'a> From<&'a ProfilesFindByIdParams> for SelectByIdParams<'a> {
    fn from(value: &'a ProfilesFindByIdParams) -> Self {
        Self {
            id: &value.id,
            user_id: &value.user_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SelectDefaultParams<'a> {
    pub user_id: &'a Uuid,
}

#[derive(Clone, Debug)]
pub struct UpdateParams<'a> {
    pub id: &'a Uuid,
    pub user_id: &'a Uuid,
    pub title: Option<&'a str>,
    pub image_url: Option<&'a str>,
}
