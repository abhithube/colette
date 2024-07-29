use anyhow::anyhow;
use chrono::{DateTime, FixedOffset};
use colette_core::{
    collections::{CollectionsCreateData, CollectionsRepository, CollectionsUpdateData, Error},
    common::{self, FindManyParams, FindOneParams},
    Collection,
};
use colette_entities::{bookmark, collection};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, JoinType, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, SelectModel, Selector, Set, TransactionError, TransactionTrait,
};
use uuid::Uuid;

pub struct CollectionsSqlRepository {
    db: DatabaseConnection,
}

impl CollectionsSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl CollectionsRepository for CollectionsSqlRepository {
    async fn find_many(&self, params: FindManyParams) -> Result<Vec<Collection>, Error> {
        collection::Entity::find()
            .select_only()
            .columns(COLLECTION_COLUMNS)
            .column_as(bookmark::Column::Id.count(), "bookmark_count")
            .join(JoinType::LeftJoin, collection::Relation::Bookmark.def())
            .filter(collection::Column::ProfileId.eq(params.profile_id))
            .filter(collection::Column::IsDefault.eq(false))
            .group_by(collection::Column::Id)
            .order_by_asc(collection::Column::Title)
            .order_by_asc(collection::Column::Id)
            .into_model::<CollectionSelect>()
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Collection::from).collect())
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn find_one(&self, params: common::FindOneParams) -> Result<Collection, Error> {
        let Some(collection) = collection_by_id(params.id, params.profile_id)
            .one(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?
        else {
            return Err(Error::NotFound(params.id));
        };

        Ok(collection.into())
    }

    async fn create(&self, data: CollectionsCreateData) -> Result<Collection, Error> {
        self.db
            .transaction::<_, Collection, Error>(|txn| {
                Box::pin(async move {
                    let new_id = Uuid::new_v4();
                    let model = collection::ActiveModel {
                        id: Set(new_id),
                        title: Set(data.title),
                        profile_id: Set(data.profile_id),
                        ..Default::default()
                    };

                    collection::Entity::insert(model)
                        .exec_without_returning(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let Some(collection) = collection_by_id(new_id, data.profile_id)
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::Unknown(anyhow!(
                            "Failed to fetch created collection"
                        )));
                    };

                    Ok(collection.into())
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
        data: CollectionsUpdateData,
    ) -> Result<Collection, Error> {
        self.db
            .transaction::<_, Collection, Error>(|txn| {
                Box::pin(async move {
                    let mut model = collection::ActiveModel {
                        id: Set(params.id),
                        ..Default::default()
                    };
                    if let Some(title) = data.title {
                        model.title = Set(title);
                    }

                    let model = collection::Entity::update(model)
                        .filter(collection::Column::ProfileId.eq(params.profile_id))
                        .filter(collection::Column::IsDefault.eq(false))
                        .exec(txn)
                        .await
                        .map_err(|e| match e {
                            DbErr::RecordNotFound(_) | DbErr::RecordNotUpdated => {
                                Error::NotFound(params.id)
                            }
                            _ => Error::Unknown(e.into()),
                        })?;

                    let Some(collection) = collection_by_id(params.id, params.profile_id)
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        if model.is_default {
                            return Err(Error::NotFound(params.id));
                        } else {
                            return Err(Error::Unknown(anyhow!(
                                "Failed to fetch updated collection"
                            )));
                        }
                    };

                    Ok(collection.into())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn delete(&self, params: common::FindOneParams) -> Result<(), Error> {
        let result = collection::Entity::delete_by_id(params.id)
            .filter(collection::Column::ProfileId.eq(params.profile_id))
            .filter(collection::Column::IsDefault.eq(false))
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
struct CollectionSelect {
    id: Uuid,
    title: String,
    profile_id: Uuid,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
    bookmark_count: Option<i64>,
}

impl From<CollectionSelect> for Collection {
    fn from(value: CollectionSelect) -> Self {
        Self {
            id: value.id,
            title: value.title,
            profile_id: value.profile_id,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
            bookmark_count: value.bookmark_count,
        }
    }
}

const COLLECTION_COLUMNS: [collection::Column; 5] = [
    collection::Column::Id,
    collection::Column::Title,
    collection::Column::ProfileId,
    collection::Column::CreatedAt,
    collection::Column::UpdatedAt,
];

fn collection_by_id(id: Uuid, profile_id: Uuid) -> Selector<SelectModel<CollectionSelect>> {
    collection::Entity::find_by_id(id)
        .select_only()
        .columns(COLLECTION_COLUMNS)
        .column_as(bookmark::Column::Id.count(), "bookmark_count")
        .join(JoinType::LeftJoin, collection::Relation::Bookmark.def())
        .filter(collection::Column::ProfileId.eq(profile_id))
        .filter(collection::Column::IsDefault.eq(false))
        .group_by(collection::Column::Id)
        .into_model::<CollectionSelect>()
}
