use colette_core::{
    User,
    user::{Error, UserFindParams, UserRepository},
};
use colette_model::users;
use sea_orm::{DatabaseConnection, EntityTrait};

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
impl UserRepository for SqliteUserRepository {
    async fn find_user(&self, params: UserFindParams) -> Result<User, Error> {
        let Some(user) = users::Entity::find_by_id(params.id.to_string())
            .one(&self.db)
            .await?
        else {
            return Err(Error::NotFound(params.id));
        };

        Ok(user.into())
    }
}
