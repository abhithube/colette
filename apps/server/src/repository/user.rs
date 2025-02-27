use colette_core::{
    User,
    common::Findable,
    user::{Error, UserFindParams, UserRepository},
};
use sea_orm::{DatabaseConnection, EntityTrait};

use super::{common::parse_date, entity};

#[derive(Debug, Clone)]
pub struct SqliteUserRepository {
    db: DatabaseConnection,
}

impl SqliteUserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteUserRepository {
    type Params = UserFindParams;
    type Output = Result<User, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let Some(user) = entity::users::Entity::find_by_id(params.id.to_string())
            .one(&self.db)
            .await?
        else {
            return Err(Error::NotFound(params.id));
        };

        Ok(user.into())
    }
}

impl UserRepository for SqliteUserRepository {}

impl From<entity::users::Model> for User {
    fn from(value: entity::users::Model) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            email: value.email,
            display_name: value.display_name,
            created_at: parse_date(&value.created_at).ok(),
            updated_at: parse_date(&value.updated_at).ok(),
        }
    }
}
