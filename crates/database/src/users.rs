use colette_core::users::{UserCreateData, UserFindOneParams};
use nanoid::nanoid;

#[derive(Debug)]
pub struct SelectByEmailParams<'a> {
    pub email: &'a str,
}

#[derive(Debug)]
pub struct InsertData<'a> {
    pub id: String,
    pub email: &'a str,
    pub password: &'a str,
}

impl<'a> From<&'a UserFindOneParams<'_>> for SelectByEmailParams<'a> {
    fn from(value: &'a UserFindOneParams<'_>) -> Self {
        Self { email: value.email }
    }
}

impl<'a> From<&'a UserCreateData<'_>> for InsertData<'a> {
    fn from(value: &'a UserCreateData<'_>) -> Self {
        Self {
            id: nanoid!(),
            email: value.email,
            password: value.password,
        }
    }
}
