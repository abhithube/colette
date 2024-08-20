use colette_core::{
    common::{Creatable, Deletable, IdParams, Paginated, Updatable},
    folder::{Error, FolderCreateData, FolderFindManyFilters, FolderRepository, FolderUpdateData},
    Folder,
};
use colette_utils::base_64;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, IntoActiveModel, SqlErr,
    TransactionError, TransactionTrait,
};
use uuid::Uuid;

use crate::queries;

pub struct FolderSqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl FolderSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Creatable for FolderSqlRepository {
    type Data = FolderCreateData;
    type Output = Result<Folder, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let model = queries::folder::insert(
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

        Ok(Folder {
            id: model.id,
            title: model.title,
            parent_id: model.parent_id,
            collection_count: Some(0),
            feed_count: Some(0),
        })
    }
}

#[async_trait::async_trait]
impl Updatable for FolderSqlRepository {
    type Params = IdParams;

    type Data = FolderUpdateData;

    type Output = Result<Folder, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        self.db
            .transaction::<_, Folder, Error>(|txn| {
                Box::pin(async move {
                    let Some(model) =
                        queries::folder::select_by_id(txn, params.id, params.profile_id)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
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
impl Deletable for FolderSqlRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let result = queries::folder::delete_by_id(&self.db, params.id, params.profile_id)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl FolderRepository for FolderSqlRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor_raw: Option<String>,
        filters: Option<FolderFindManyFilters>,
    ) -> Result<Paginated<Folder>, Error> {
        find(&self.db, None, profile_id, limit, cursor_raw, filters).await
    }

    async fn find(&self, params: IdParams) -> Result<Folder, Error> {
        find_by_id(&self.db, params).await
    }
}

async fn find<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor_raw: Option<String>,
    filters: Option<FolderFindManyFilters>,
) -> Result<Paginated<Folder>, Error> {
    let mut cursor = Cursor::default();
    if let Some(raw) = cursor_raw.as_deref() {
        cursor = base_64::decode::<Cursor>(raw)?;
    }

    let mut folders =
        queries::folder::select(db, id, profile_id, limit.map(|e| e + 1), cursor, filters)
            .await
            .map(|e| e.into_iter().map(Folder::from).collect::<Vec<_>>())
            .map_err(|e| Error::Unknown(e.into()))?;
    let mut cursor: Option<String> = None;

    if let Some(limit) = limit {
        let limit = limit as usize;
        if folders.len() > limit {
            folders = folders.into_iter().take(limit).collect();

            if let Some(last) = folders.last() {
                let c = Cursor {
                    title: last.title.to_owned(),
                };
                let encoded = base_64::encode(&c)?;

                cursor = Some(encoded);
            }
        }
    }

    Ok(Paginated::<Folder> {
        cursor,
        data: folders,
    })
}

async fn find_by_id<Db: ConnectionTrait>(db: &Db, params: IdParams) -> Result<Folder, Error> {
    let folders = find(db, Some(params.id), params.profile_id, Some(1), None, None).await?;

    folders
        .data
        .first()
        .cloned()
        .ok_or(Error::NotFound(params.id))
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    pub title: String,
}
