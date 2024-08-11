use colette_core::{
    common::{Paginated, PAGINATION_LIMIT},
    profiles::{Error, ProfilesCreateData, ProfilesRepository, ProfilesUpdateData, StreamProfile},
    Profile,
};
use colette_entities::{profile, profile_feed};
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, JoinType,
    ModelTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set, SqlErr, TransactionError,
    TransactionTrait,
};
use uuid::Uuid;

use crate::{utils, SqlRepository};

#[async_trait::async_trait]
impl ProfilesRepository for SqlRepository {
    async fn find_many_profiles(
        &self,
        user_id: Uuid,
        limit: Option<u64>,
        cursor_raw: Option<String>,
    ) -> Result<Paginated<Profile>, Error> {
        let mut cursor = Cursor::default();
        if let Some(raw) = cursor_raw.as_deref() {
            cursor = utils::decode_cursor::<Cursor>(raw).map_err(|e| Error::Unknown(e.into()))?;
        }

        let mut query = profile::Entity::find()
            .filter(profile::Column::UserId.eq(user_id))
            .order_by_asc(profile::Column::Title)
            .cursor_by(profile::Column::Title);

        query.after(cursor.title);
        if let Some(limit) = limit {
            query.first(limit);
        }

        let mut profiles = query
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Profile::from).collect::<Vec<_>>())
            .map_err(|e| Error::Unknown(e.into()))?;
        let mut cursor: Option<String> = None;

        if profiles.len() > PAGINATION_LIMIT {
            profiles = profiles.into_iter().take(PAGINATION_LIMIT).collect();

            if let Some(last) = profiles.last() {
                let c = Cursor {
                    title: last.title.to_owned(),
                };
                let encoded = utils::encode_cursor(&c).map_err(|e| Error::Unknown(e.into()))?;

                cursor = Some(encoded);
            }
        }

        Ok(Paginated::<Profile> {
            cursor,
            data: profiles,
        })
    }

    async fn find_one_profile(&self, id: Option<Uuid>, user_id: Uuid) -> Result<Profile, Error> {
        match id {
            Some(id) => find_by_id(&self.db, id, user_id).await,
            None => {
                let Some(profile) = profile::Entity::find()
                    .filter(profile::Column::UserId.eq(user_id))
                    .filter(profile::Column::IsDefault.eq(true))
                    .one(&self.db)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
                else {
                    return Err(Error::Unknown(anyhow::anyhow!(
                        "couldn't find default profile"
                    )));
                };

                Ok(profile.into())
            }
        }
    }

    async fn create_profile(&self, data: ProfilesCreateData) -> Result<Profile, Error> {
        let model = profile::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(data.title.clone()),
            image_url: Set(data.image_url),
            user_id: Set(data.user_id),
            ..Default::default()
        };

        let profile = profile::Entity::insert(model)
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| match e.sql_err() {
                Some(SqlErr::UniqueConstraintViolation(_)) => Error::Conflict(data.title),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(profile.into())
    }

    async fn update_profile(
        &self,
        id: Uuid,
        user_id: Uuid,
        data: ProfilesUpdateData,
    ) -> Result<Profile, Error> {
        self.db
            .transaction::<_, colette_core::Profile, Error>(|txn| {
                Box::pin(async move {
                    let Some(mut model) = profile::Entity::find_by_id(id)
                        .filter(profile::Column::UserId.eq(user_id))
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(id));
                    };
                    let mut active_model = model.clone().into_active_model();

                    if let Some(title) = data.title {
                        active_model.title.set_if_not_equals(title);
                    }
                    if data.image_url.is_some() {
                        active_model.image_url.set_if_not_equals(data.image_url);
                    }

                    if active_model.is_changed() {
                        model = active_model
                            .update(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    Ok(model.into())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn delete_profile(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        self.db
            .transaction::<_, (), Error>(|txn| {
                Box::pin(async move {
                    let Some(profile) = profile::Entity::find_by_id(id)
                        .filter(profile::Column::UserId.eq(user_id))
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(id));
                    };

                    if profile.is_default {
                        return Err(Error::DeletingDefault);
                    }

                    profile
                        .delete(txn)
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

    async fn stream_profiles(
        &self,
        feed_id: i32,
    ) -> Result<BoxStream<Result<StreamProfile, Error>>, Error> {
        profile::Entity::find()
            .join(JoinType::InnerJoin, profile::Relation::ProfileFeed.def())
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
pub struct StreamSelect {
    pub id: Uuid,
}

impl From<StreamSelect> for StreamProfile {
    fn from(value: StreamSelect) -> Self {
        Self { id: value.id }
    }
}

async fn find_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    user_id: Uuid,
) -> Result<colette_core::Profile, Error> {
    let Some(profile) = profile::Entity::find_by_id(id)
        .filter(profile::Column::UserId.eq(user_id))
        .one(db)
        .await
        .map_err(|e| Error::Unknown(e.into()))?
    else {
        return Err(Error::NotFound(id));
    };

    Ok(profile.into())
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
struct Cursor {
    pub title: String,
}
