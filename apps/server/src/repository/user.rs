use colette_core::{
    User,
    common::Findable,
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
impl Findable for SqliteUserRepository {
    type Params = UserFindParams;
    type Output = Result<User, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let Some(user) = users::Entity::find_by_id(params.id.to_string())
            .one(&self.db)
            .await?
        else {
            return Err(Error::NotFound(params.id));
        };

        Ok(user.into())
    }
}

impl UserRepository for SqliteUserRepository {}
