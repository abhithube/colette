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

impl<'a> From<&UserFindOneParams<'a>> for SelectByEmailParams<'a> {
    fn from(value: &UserFindOneParams<'a>) -> Self {
        Self { email: value.email }
    }
}

impl<'a> From<&UserCreateData<'a>> for InsertData<'a> {
    fn from(value: &UserCreateData<'a>) -> Self {
        Self {
            id: nanoid!(),
            email: value.email,
            password: value.password,
        }
    }
}
