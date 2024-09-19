use colette_core::{
    collection::{CollectionCreateData, CollectionRepository, CollectionUpdateData, Cursor, Error},
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    Collection,
};
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, IntoActiveModel, SqlErr,
    TransactionError, TransactionTrait,
};
use uuid::Uuid;

use crate::query;

pub struct CollectionSqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl CollectionSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for CollectionSqlRepository {
    type Params = IdParams;
    type Output = Result<Collection, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        find_by_id(&self.db, params).await
    }
}

#[async_trait::async_trait]
impl Creatable for CollectionSqlRepository {
    type Data = CollectionCreateData;
    type Output = Result<Collection, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let model = query::collection::insert(
            &self.db,
            Uuid::new_v4(),
            data.title.clone(),
            data.parent_id,
            data.profile_id,
        )
        .await
        .map_err(|e| match e.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => Error::Conflict(data.title),
            _ => Error::Unknown(e.into()),
        })?;

        Ok(Collection {
            id: model.id,
            title: model.title,
            parent_id: model.parent_id,
            bookmark_count: Some(0),
        })
    }
}

#[async_trait::async_trait]
impl Updatable for CollectionSqlRepository {
    type Params = IdParams;
    type Data = CollectionUpdateData;
    type Output = Result<Collection, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        self.db
            .transaction::<_, Collection, Error>(|txn| {
                Box::pin(async move {
                    let Some(model) =
                        query::collection::select_by_id(txn, params.id, params.profile_id)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
                    };
                    let mut active_model = model.into_active_model();

                    if let Some(title) = data.title {
                        active_model.title.set_if_not_equals(title);
                    }
                    if let Some(parent_id) = data.parent_id {
                        active_model.parent_id.set_if_not_equals(parent_id);
                    }

                    if active_model.is_changed() {
                        active_model
                            .update(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    find_by_id(txn, params).await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }
}

#[async_trait::async_trait]
impl Deletable for CollectionSqlRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let result = query::collection::delete_by_id(&self.db, params.id, params.profile_id)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl CollectionRepository for CollectionSqlRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
    ) -> Result<Vec<Collection>, Error> {
        find(&self.db, None, profile_id, limit, cursor).await
    }
}

pub(crate) async fn find<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
) -> Result<Vec<Collection>, Error> {
    let collections = query::collection::select(db, id, profile_id, limit, cursor)
        .await
        .map(|e| e.into_iter().map(Collection::from).collect::<Vec<_>>())
        .map_err(|e| Error::Unknown(e.into()))?;

    Ok(collections)
}

async fn find_by_id<Db: ConnectionTrait>(db: &Db, params: IdParams) -> Result<Collection, Error> {
    let mut collections = find(db, Some(params.id), params.profile_id, None, None).await?;
    if collections.is_empty() {
        return Err(Error::NotFound(params.id));
    }

    Ok(collections.swap_remove(0))
}
