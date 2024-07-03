use colette_core::profiles::{
    ProfileCreateData, ProfileFindByIdParams, ProfileFindManyParams, ProfileUpdateData,
};
use nanoid::nanoid;

#[derive(Debug)]
pub struct SelectManyParams<'a> {
    pub user_id: &'a str,
}

#[derive(Debug)]
pub struct SelectByIdParams<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
}

#[derive(Debug)]
pub struct SelectDefaultParams<'a> {
    pub user_id: &'a str,
}

#[derive(Debug)]
pub struct InsertData<'a> {
    pub id: String,
    pub title: &'a str,
    pub image_url: Option<&'a str>,
    pub is_default: bool,
    pub user_id: &'a str,
}

#[derive(Debug)]
pub struct UpdateData<'a> {
    pub title: Option<&'a str>,
    pub image_url: Option<&'a str>,
}

impl<'a> From<&'a ProfileFindManyParams> for SelectManyParams<'a> {
    fn from(value: &'a ProfileFindManyParams) -> Self {
        Self {
            user_id: value.user_id.as_str(),
        }
    }
}

impl<'a> From<&'a ProfileFindByIdParams> for SelectByIdParams<'a> {
    fn from(value: &'a ProfileFindByIdParams) -> Self {
        Self {
            id: value.id.as_str(),
            user_id: value.user_id.as_str(),
        }
    }
}

impl<'a> From<&'a ProfileCreateData> for InsertData<'a> {
    fn from(value: &'a ProfileCreateData) -> Self {
        Self {
            id: nanoid!(),
            title: value.title.as_str(),
            image_url: value.image_url.as_deref(),
            is_default: false,
            user_id: value.user_id.as_str(),
        }
    }
}

impl<'a> From<&'a ProfileUpdateData> for UpdateData<'a> {
    fn from(value: &'a ProfileUpdateData) -> Self {
        Self {
            title: value.title.as_deref(),
            image_url: value.image_url.as_deref(),
        }
    }
}
