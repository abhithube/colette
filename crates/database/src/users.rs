use colette_core::users::{UserCreateData, UserFindOneParams};
use nanoid::nanoid;

#[derive(Debug)]
pub struct SelectByEmailParams<'a> {
    pub email: &'a str,
}

impl<'a> From<&'a UserFindOneParams> for SelectByEmailParams<'a> {
    fn from(value: &'a UserFindOneParams) -> Self {
        Self {
            email: &value.email,
        }
    }
}

#[derive(Debug)]
pub struct InsertData<'a> {
    pub id: String,
    pub email: &'a str,
    pub password: &'a str,
}

impl<'a> From<&'a UserCreateData> for InsertData<'a> {
    fn from(value: &'a UserCreateData) -> Self {
        Self {
            id: nanoid!(),
            email: &value.email,
            password: &value.password,
        }
    }
}
