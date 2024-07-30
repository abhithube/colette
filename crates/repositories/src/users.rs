use anyhow::anyhow;
use colette_core::{
    users::{Error, NotFoundError, UsersCreateData, UsersFindOneParams, UsersRepository},
    User,
};
use colette_entities::{profile, user};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, SqlErr, TransactionTrait,
};
use uuid::Uuid;

pub struct UsersSqlRepository {
    db: DatabaseConnection,
}

impl UsersSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl UsersRepository for UsersSqlRepository {
    async fn find_one(&self, params: UsersFindOneParams) -> Result<User, Error> {
        let Some(user) = user::Entity::find()
            .filter(user::Column::Email.eq(&params.email))
            .one(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?
        else {
            return Err(Error::NotFound(NotFoundError::Email(params.email)));
        };

        Ok(user.into())
    }

    async fn create(&self, data: UsersCreateData) -> Result<User, Error> {
        self.db
            .transaction::<_, User, Error>(|txn| {
                Box::pin(async move {
                    let new_user_id = Uuid::new_v4();
                    let user = user::ActiveModel {
                        id: Set(new_user_id),
                        email: Set(data.email.clone()),
                        password: Set(data.password),
                        ..Default::default()
                    };

                    user::Entity::insert(user)
                        .exec_without_returning(txn)
                        .await
                        .map_err(|e| match e.sql_err() {
                            Some(SqlErr::UniqueConstraintViolation(_)) => {
                                Error::Conflict(data.email)
                            }
                            _ => Error::Unknown(e.into()),
                        })?;

                    let profile_model = profile::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        title: Set("Default".to_owned()),
                        is_default: Set(true),
                        user_id: Set(new_user_id),
                        ..Default::default()
                    };

                    profile::Entity::insert(profile_model)
                        .exec_without_returning(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let Some(user) = user::Entity::find_by_id(new_user_id)
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::Unknown(anyhow!("Failed to fetch created user")));
                    };

                    Ok(user.into())
                })
            })
            .await
            .map_err(|e| Error::Unknown(e.into()))
    }
}
