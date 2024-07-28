use anyhow::anyhow;
use colette_core::{
    users::{Error, NotFoundError, UsersCreateData, UsersFindOneParams, UsersRepository},
    User,
};
use colette_entities::{collections, profiles, users};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, SelectModel, Selector,
    Set, SqlErr, SqlxPostgresConnector, TransactionTrait,
};
use sqlx::{
    types::chrono::{DateTime, FixedOffset},
    PgPool,
};
use uuid::Uuid;

pub struct UsersPostgresRepository {
    db: DatabaseConnection,
}

impl UsersPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            db: SqlxPostgresConnector::from_sqlx_postgres_pool(pool),
        }
    }
}

#[async_trait::async_trait]
impl UsersRepository for UsersPostgresRepository {
    async fn find_one(&self, params: UsersFindOneParams) -> Result<User, Error> {
        let Some(user) = users::Entity::find()
            .select_only()
            .columns(USER_COLUMNS)
            .filter(users::Column::Email.eq(&params.email))
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
                    let user = users::ActiveModel {
                        id: Set(new_user_id),
                        email: Set(data.email.clone()),
                        password: Set(data.password),
                        ..Default::default()
                    };

                    users::Entity::insert(user)
                        .exec_without_returning(txn)
                        .await
                        .map_err(|e| match e.sql_err() {
                            Some(SqlErr::UniqueConstraintViolation(_)) => {
                                Error::Conflict(data.email)
                            }
                            _ => Error::Unknown(e.into()),
                        })?;

                    let new_profile_id = Uuid::new_v4();
                    let profile_model = profiles::ActiveModel {
                        id: Set(new_profile_id),
                        title: Set("Default".to_owned()),
                        is_default: Set(true),
                        user_id: Set(new_user_id),
                        ..Default::default()
                    };

                    profiles::Entity::insert(profile_model)
                        .exec_without_returning(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let collection_model = collections::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        title: Set("Default".to_owned()),
                        is_default: Set(true),
                        profile_id: Set(new_profile_id),
                        ..Default::default()
                    };

                    collections::Entity::insert(collection_model)
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

const USER_COLUMNS: [users::Column; 5] = [
    users::Column::Id,
    users::Column::Email,
    users::Column::Password,
    users::Column::CreatedAt,
    users::Column::UpdatedAt,
];

fn user_by_id(id: Uuid) -> Selector<SelectModel<UserSelect>> {
    users::Entity::find_by_id(id)
        .select_only()
        .columns(USER_COLUMNS)
        .into_model::<UserSelect>()
}
