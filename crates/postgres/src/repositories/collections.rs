use anyhow::anyhow;
use colette_core::{
    collections::{CollectionsCreateData, CollectionsRepository, CollectionsUpdateData, Error},
    common::{self, FindManyParams, FindOneParams},
    Collection,
};
use colette_entities::{bookmarks, collections};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, JoinType, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, SelectModel, Selector, Set, TransactionError, TransactionTrait,
};
use sqlx::types::chrono::{DateTime, FixedOffset};
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
        collections::Entity::find()
            .select_only()
            .columns(COLLECTION_COLUMNS)
            .column_as(bookmarks::Column::Id.count(), "count")
            .join(JoinType::LeftJoin, collections::Relation::Bookmarks.def())
            .filter(collections::Column::ProfileId.eq(params.profile_id))
            .order_by_asc(collections::Column::Title)
            .order_by_asc(collections::Column::Id)
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
                    let model = collections::ActiveModel {
                        id: Set(new_id),
                        title: Set(data.title),
                        profile_id: Set(data.profile_id),
                        ..Default::default()
                    };

                    collections::Entity::insert(model)
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
                    let mut model = collections::ActiveModel {
                        id: Set(params.id),
                        ..Default::default()
                    };
                    if let Some(title) = data.title {
                        model.title = Set(title);
                    }

                    collections::Entity::update(model)
                        .filter(collections::Column::ProfileId.eq(params.profile_id))
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
                        return Err(Error::Unknown(anyhow!(
                            "Failed to fetch updated collection"
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

    async fn delete(&self, params: common::FindOneParams) -> Result<(), Error> {
        let result = collections::Entity::delete_by_id(params.id)
            .filter(collections::Column::ProfileId.eq(params.profile_id))
            .exec(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[derive(sea_orm::FromQueryResult)]
struct CollectionSelect {
    id: Uuid,
    title: String,
    profile_id: Uuid,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
    bookmark_count: Option<u64>,
}

impl From<CollectionSelect> for Collection {
    fn from(value: CollectionSelect) -> Self {
        Self {
            id: value.id,
            title: value.title,
            profile_id: value.profile_id,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
            bookmark_count: value.bookmark_count.map(|e| e as i64),
        }
    }
}

const COLLECTION_COLUMNS: [collections::Column; 5] = [
    collections::Column::Id,
    collections::Column::Title,
    collections::Column::ProfileId,
    collections::Column::CreatedAt,
    collections::Column::UpdatedAt,
];

fn collection_by_id(id: Uuid, profile_id: Uuid) -> Selector<SelectModel<CollectionSelect>> {
    collections::Entity::find_by_id(id)
        .select_only()
        .columns(COLLECTION_COLUMNS)
        .column_as(bookmarks::Column::Id.count(), "count")
        .join(JoinType::LeftJoin, collections::Relation::Bookmarks.def())
        .filter(collections::Column::ProfileId.eq(profile_id))
        .into_model::<CollectionSelect>()
}
