use colette_core::{
    collection::{CollectionCreateData, CollectionRepository, CollectionUpdateData, Error},
    common::{Creatable, Deletable, Findable, IdParams, Paginated, Updatable},
    Collection,
};
use colette_utils::base_64;
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
            data.folder_id,
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
            folder_id: model.folder_id,
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
                    if let Some(folder_id) = data.folder_id {
                        active_model.folder_id.set_if_not_equals(folder_id);
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
        cursor_raw: Option<String>,
    ) -> Result<Paginated<Collection>, Error> {
        find(&self.db, None, profile_id, limit, cursor_raw).await
    }
}

async fn find<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor_raw: Option<String>,
) -> Result<Paginated<Collection>, Error> {
    let mut cursor = Cursor::default();
    if let Some(raw) = cursor_raw.as_deref() {
        cursor = base_64::decode::<Cursor>(raw)?;
    }

    let mut collections =
        query::collection::select(db, id, profile_id, limit.map(|e| e + 1), cursor)
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

async fn find_by_id<Db: ConnectionTrait>(db: &Db, params: IdParams) -> Result<Collection, Error> {
    let collections = find(db, Some(params.id), params.profile_id, None, None).await?;

    collections
        .data
        .first()
        .cloned()
        .ok_or(Error::NotFound(params.id))
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    pub title: String,
}