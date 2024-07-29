use anyhow::anyhow;
use chrono::{DateTime, FixedOffset};
use colette_core::{
    common::{self, FindManyParams, FindOneParams},
    tags::{Error, Tag, TagsCreateData, TagsRepository, TagsUpdateData},
    TagDetails,
};
use colette_entities::tags;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QuerySelect, SelectModel,
    Selector, Set, TransactionError, TransactionTrait,
};
use uuid::Uuid;

pub struct TagsSqlRepository {
    db: DatabaseConnection,
}

impl TagsSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl TagsRepository for TagsSqlRepository {
    async fn find_many(&self, params: FindManyParams) -> Result<Vec<Tag>, Error> {
        tags::Entity::find()
            .select_only()
            .columns([tags::Column::Id, tags::Column::Title])
            .filter(tags::Column::ProfileId.eq(params.profile_id))
            .into_model::<TagSelect>()
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Tag::from).collect())
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn find_one(&self, params: common::FindOneParams) -> Result<TagDetails, Error> {
        let Some(tag) = tag_by_id(params.id, params.profile_id)
            .one(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?
        else {
            return Err(Error::NotFound(params.id));
        };

        Ok(tag.into())
    }

    async fn create(&self, data: TagsCreateData) -> Result<TagDetails, Error> {
        self.db
            .transaction::<_, TagDetails, Error>(|txn| {
                Box::pin(async move {
                    let new_id = Uuid::new_v4();
                    let model = tags::ActiveModel {
                        id: Set(new_id),
                        title: Set(data.title),
                        profile_id: Set(data.profile_id),
                        ..Default::default()
                    };

                    tags::Entity::insert(model)
                        .exec_without_returning(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let Some(tag) = tag_by_id(new_id, data.profile_id)
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::Unknown(anyhow!("Failed to fetch created tag")));
                    };

                    Ok(tag.into())
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
        params: FindOneParams,
        data: TagsUpdateData,
    ) -> Result<TagDetails, Error> {
        self.db
            .transaction::<_, TagDetails, Error>(|txn| {
                Box::pin(async move {
                    let mut model = tags::ActiveModel {
                        id: Set(params.id),
                        ..Default::default()
                    };
                    if let Some(title) = data.title {
                        model.title = Set(title);
                    }

                    tags::Entity::update(model)
                        .filter(tags::Column::ProfileId.eq(params.profile_id))
                        .exec(txn)
                        .await
                        .map_err(|e| match e {
                            DbErr::RecordNotFound(_) | DbErr::RecordNotUpdated => {
                                Error::NotFound(params.id)
                            }
                            _ => Error::Unknown(e.into()),
                        })?;

                    let Some(tag) = tag_by_id(params.id, params.profile_id)
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::Unknown(anyhow!("Failed to fetch updated tag")));
                    };

                    Ok(tag.into())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn delete(&self, params: common::FindOneParams) -> Result<(), Error> {
        let result = tags::Entity::delete_by_id(params.id)
            .filter(tags::Column::ProfileId.eq(params.profile_id))
            .exec(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
struct TagSelect {
    id: Uuid,
    title: String,
}

impl From<TagSelect> for Tag {
    fn from(value: TagSelect) -> Self {
        Self {
            id: value.id,
            title: value.title,
        }
    }
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
struct TagDetailsSelect {
    id: Uuid,
    title: String,
    profile_id: Uuid,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
}

impl From<TagDetailsSelect> for TagDetails {
    fn from(value: TagDetailsSelect) -> Self {
        Self {
            id: value.id,
            title: value.title,
            profile_id: value.profile_id,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
        }
    }
}

fn tag_by_id(id: Uuid, profile_id: Uuid) -> Selector<SelectModel<TagDetailsSelect>> {
    tags::Entity::find_by_id(id)
        .select_only()
        .columns([
            tags::Column::Id,
            tags::Column::Title,
            tags::Column::ProfileId,
            tags::Column::CreatedAt,
            tags::Column::UpdatedAt,
        ])
        .filter(tags::Column::ProfileId.eq(profile_id))
        .into_model::<TagDetailsSelect>()
}
