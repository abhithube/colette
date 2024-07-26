use colette_core::profiles::{ProfilesFindByIdParams, ProfilesFindManyParams};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SelectManyParams {
    pub user_id: Uuid,
}

impl<'a> From<&'a ProfilesFindManyParams> for SelectManyParams {
    fn from(value: &'a ProfilesFindManyParams) -> Self {
        Self {
            user_id: value.user_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SelectByIdParams {
    pub id: Uuid,
    pub user_id: Uuid,
}

impl<'a> From<&'a ProfilesFindByIdParams> for SelectByIdParams {
    fn from(value: &'a ProfilesFindByIdParams) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SelectDefaultParams {
    pub user_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct UpdateParams<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: Option<&'a str>,
    pub image_url: Option<&'a str>,
}
