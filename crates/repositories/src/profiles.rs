use anyhow::anyhow;
use chrono::{DateTime, FixedOffset};
use colette_core::{
    profiles::{
        Error, ProfilesCreateData, ProfilesFindByIdParams, ProfilesFindManyParams,
        ProfilesFindOneParams, ProfilesRepository, ProfilesUpdateData, StreamProfile,
    },
    Profile,
};
use colette_entities::{profile, profile_feed};
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, JoinType, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, Set, TransactionError, TransactionTrait,
};
use uuid::Uuid;

pub struct ProfilesSqlRepository {
    db: DatabaseConnection,
}

impl ProfilesSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl ProfilesRepository for ProfilesSqlRepository {
    async fn find_many(&self, params: ProfilesFindManyParams) -> Result<Vec<Profile>, Error> {
        profile::Entity::find()
            .filter(profile::Column::UserId.eq(params.user_id))
            .order_by_asc(profile::Column::Title)
            .order_by_asc(profile::Column::Id)
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Profile::from).collect())
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn find_one(&self, params: ProfilesFindOneParams) -> Result<Profile, Error> {
        match params {
            ProfilesFindOneParams::ById(params) => {
                let Some(profile) = profile::Entity::find_by_id(params.id)
                    .filter(profile::Column::UserId.eq(params.user_id))
                    .one(&self.db)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
                else {
                    return Err(Error::NotFound(params.id));
                };

                Ok(profile.into())
            }
            ProfilesFindOneParams::Default { user_id } => {
                let Some(profile) = profile::Entity::find()
                    .filter(profile::Column::IsDefault.eq(true))
                    .filter(profile::Column::UserId.eq(user_id))
                    .into_model::<ProfileSelect>()
                    .one(&self.db)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
                else {
                    return Err(Error::Unknown(anyhow!("Failed to fetch default profile")));
                };

                Ok(profile.into())
            }
        }
    }

    async fn create(&self, data: ProfilesCreateData) -> Result<Profile, Error> {
        self.db
            .transaction::<_, Profile, Error>(|txn| {
                Box::pin(async move {
                    let new_id = Uuid::new_v4();
                    let profile_model = profile::ActiveModel {
                        id: Set(new_id),
                        title: Set(data.title),
                        image_url: Set(data.image_url),
                        user_id: Set(data.user_id),
                        ..Default::default()
                    };

                    profile::Entity::insert(profile_model)
                        .exec_without_returning(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let Some(profile) = profile::Entity::find_by_id(new_id)
                        .filter(profile::Column::UserId.eq(data.user_id))
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::Unknown(anyhow!("Failed to fetch created profile")));
                    };

                    Ok(profile.into())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn update(
        &self,
        params: ProfilesFindByIdParams,
        data: ProfilesUpdateData,
    ) -> Result<Profile, Error> {
        self.db
            .transaction::<_, Profile, Error>(|txn| {
                Box::pin(async move {
                    let mut model = profile::ActiveModel {
                        id: Set(params.id),
                        ..Default::default()
                    };
                    if let Some(title) = data.title {
                        model.title = Set(title);
                    }
                    if data.image_url.is_some() {
                        model.image_url = Set(data.image_url)
                    }

                    let profile = profile::Entity::update(model)
                        .filter(profile::Column::UserId.eq(params.user_id))
                        .exec(txn)
                        .await
                        .map_err(|e| match e {
                            DbErr::RecordNotFound(_) | DbErr::RecordNotUpdated => {
                                Error::NotFound(params.id)
                            }
                            _ => Error::Unknown(e.into()),
                        })?;

                    Ok(profile.into())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn delete(&self, params: ProfilesFindByIdParams) -> Result<(), Error> {
        self.db
            .transaction::<_, (), Error>(|txn| {
                Box::pin(async move {
                    let Some(profile) = profile::Entity::find_by_id(params.id)
                        .select_only()
                        .column(profile::Column::IsDefault)
                        .filter(profile::Column::UserId.eq(params.user_id))
                        .into_model::<ProfileDelete>()
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
                    };

                    if profile.is_default {
                        return Err(Error::DeletingDefault);
                    }

                    profile::Entity::delete_by_id(params.id)
                        .exec(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn stream(&self, feed_id: i32) -> Result<BoxStream<Result<StreamProfile, Error>>, Error> {
        profile::Entity::find()
            .select_only()
            .column(profile::Column::Id)
            .join(JoinType::Join, profile::Relation::ProfileFeed.def())
            .filter(profile_feed::Column::FeedId.eq(feed_id))
            .into_model::<StreamSelect>()
            .stream(&self.db)
            .await
            .map(|e| {
                e.map(|e| {
                    e.map(StreamProfile::from)
                        .map_err(|e| Error::Unknown(e.into()))
                })
                .map_err(|e| Error::Unknown(e.into()))
                .boxed()
            })
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
struct ProfileSelect {
    id: Uuid,
    title: String,
    image_url: Option<String>,
    user_id: Uuid,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
}

impl From<ProfileSelect> for Profile {
    fn from(value: ProfileSelect) -> Self {
        Self {
            id: value.id,
            title: value.title,
            image_url: value.image_url,
            user_id: value.user_id,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
        }
    }
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
struct ProfileDelete {
    is_default: bool,
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
pub struct StreamSelect {
    pub id: Uuid,
}

impl From<StreamSelect> for StreamProfile {
    fn from(value: StreamSelect) -> Self {
        Self { id: value.id }
    }
}
