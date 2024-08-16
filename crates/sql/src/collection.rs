use colette_core::{
    collection::{CollectionCreateData, CollectionRepository, CollectionUpdateData, Error},
    common::Paginated,
    Collection,
};
use colette_entities::{collection, profile_bookmark, PartialCollection};
use colette_utils::base_64;
use sea_orm::{
    sea_query::{Alias, Expr},
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, IntoActiveModel,
    JoinType, QueryFilter, QuerySelect, RelationTrait, Set, SqlErr, TransactionError,
    TransactionTrait,
};
use uuid::Uuid;

use crate::SqlRepository;

#[async_trait::async_trait]
impl CollectionRepository for SqlRepository {
    async fn find_many_collections(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor_raw: Option<String>,
    ) -> Result<Paginated<Collection>, Error> {
        find(&self.db, None, profile_id, limit, cursor_raw).await
    }

    async fn find_one_collection(&self, id: Uuid, profile_id: Uuid) -> Result<Collection, Error> {
        find_by_id(&self.db, id, profile_id).await
    }

    async fn create_collection(&self, data: CollectionCreateData) -> Result<Collection, Error> {
        let model = collection::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(data.title.clone()),
            profile_id: Set(data.profile_id),
            ..Default::default()
        };

        let collection = collection::Entity::insert(model)
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| match e.sql_err() {
                Some(SqlErr::UniqueConstraintViolation(_)) => Error::Conflict(data.title),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(Collection {
            id: collection.id,
            title: collection.title,
            folder_id: collection.folder_id,
            bookmark_count: Some(0),
        })
    }

    async fn update_collection(
        &self,
        id: Uuid,
        profile_id: Uuid,
        data: CollectionUpdateData,
    ) -> Result<Collection, Error> {
        self.db
            .transaction::<_, Collection, Error>(|txn| {
                Box::pin(async move {
                    let Some(model) = collection::Entity::find_by_id(id)
                        .filter(collection::Column::ProfileId.eq(profile_id))
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(id));
                    };
                    let mut active_model = model.into_active_model();

                    if let Some(title) = data.title {
                        active_model.title.set_if_not_equals(title);
                    }

                    if active_model.is_changed() {
                        active_model
                            .update(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    find_by_id(txn, id, profile_id).await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn delete_collection(&self, id: Uuid, profile_id: Uuid) -> Result<(), Error> {
        let result = collection::Entity::delete_by_id(id)
            .filter(collection::Column::ProfileId.eq(profile_id))
            .exec(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(id));
        }

        Ok(())
    }
}

async fn find<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor_raw: Option<String>,
) -> Result<Paginated<Collection>, Error> {
    let mut conditions = Condition::all().add(collection::Column::ProfileId.eq(profile_id));
    if let Some(id) = id {
        conditions = conditions.add(collection::Column::Id.eq(id));
    }

    let mut cursor = Cursor::default();
    if let Some(raw) = cursor_raw.as_deref() {
        cursor = base_64::decode::<Cursor>(raw)?;
    }

    let mut query = collection::Entity::find()
        .expr_as(
            Expr::col((Alias::new("pb"), profile_bookmark::Column::CollectionId)).count(),
            "bookmark_count",
        )
        .join_as(
            JoinType::LeftJoin,
            collection::Relation::ProfileBookmark.def(),
            Alias::new("pb"),
        )
        .group_by(collection::Column::Id)
        .filter(conditions)
        .cursor_by(collection::Column::Title);
    query.after(cursor.title);
    if let Some(limit) = limit {
        query.first(limit + 1);
    }

    let mut collections = query
        .into_model::<PartialCollection>()
        .all(db)
        .await
        .map(|e| e.into_iter().map(Collection::from).collect::<Vec<_>>())
        .map_err(|e| Error::Unknown(e.into()))?;
    let mut cursor: Option<String> = None;

    if let Some(limit) = limit {
        let limit = limit as usize;
        if collections.len() > limit {
            collections = collections.into_iter().take(limit).collect();

            if let Some(last) = collections.last() {
                let c = Cursor {
                    title: last.title.to_owned(),
                };
                let encoded = base_64::encode(&c)?;

                cursor = Some(encoded);
            }
        }
    }

    Ok(Paginated::<Collection> {
        cursor,
        data: collections,
    })
}

async fn find_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    profile_id: Uuid,
) -> Result<Collection, Error> {
    let collections = find(db, Some(id), profile_id, Some(1), None).await?;

    collections.data.first().cloned().ok_or(Error::NotFound(id))
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
struct Cursor {
    pub title: String,
}
