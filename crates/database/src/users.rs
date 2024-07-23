use colette_core::users::UsersFindOneParams;

#[derive(Clone, Debug)]
pub struct SelectByEmailParams<'a> {
    pub email: &'a str,
}

impl<'a> From<&'a UsersFindOneParams> for SelectByEmailParams<'a> {
    fn from(value: &'a UsersFindOneParams) -> Self {
        Self {
            email: &value.email,
        }
    }
}
