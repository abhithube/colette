use anyhow::anyhow;
use chrono::{DateTime, FixedOffset};
use colette_core::{
    users::{Error, NotFoundError, UsersCreateData, UsersFindOneParams, UsersRepository},
    User,
};
use colette_entities::{profile, user};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, SelectModel, Selector,
    Set, SqlErr, TransactionTrait,
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
            .select_only()
            .columns(USER_COLUMNS)
            .filter(user::Column::Email.eq(&params.email))
            .into_model::<UserSelect>()
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

                    let new_profile_id = Uuid::new_v4();
                    let profile_model = profile::ActiveModel {
                        id: Set(new_profile_id),
                        title: Set("Default".to_owned()),
                        is_default: Set(true),
                        user_id: Set(new_user_id),
                        ..Default::default()
                    };

                    profile::Entity::insert(profile_model)
                        .exec_without_returning(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let Some(user) = user_by_id(new_user_id)
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

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
struct UserSelect {
    id: Uuid,
    email: String,
    password: String,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
}

impl From<UserSelect> for User {
    fn from(value: UserSelect) -> Self {
        Self {
            id: value.id,
            email: value.email,
            password: value.password,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
        }
    }
}

const USER_COLUMNS: [user::Column; 5] = [
    user::Column::Id,
    user::Column::Email,
    user::Column::Password,
    user::Column::CreatedAt,
    user::Column::UpdatedAt,
];

fn user_by_id(id: Uuid) -> Selector<SelectModel<UserSelect>> {
    user::Entity::find_by_id(id)
        .select_only()
        .columns(USER_COLUMNS)
        .into_model::<UserSelect>()
}
