use colette_core::users::UserFindOneParams;

#[derive(Clone, Debug)]
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
