use colette_core::{
    common::Paginated,
    folder::{Error, FolderCreateData, FolderFindManyFilters, FolderRepository, FolderUpdateData},
    Folder,
};
use colette_utils::base_64;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, IntoActiveModel, SqlErr, TransactionError, TransactionTrait,
};
use uuid::Uuid;

use crate::{queries, SqlRepository};

#[async_trait::async_trait]
impl FolderRepository for SqlRepository {
    async fn find_many_folders(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor_raw: Option<String>,
        filters: Option<FolderFindManyFilters>,
    ) -> Result<Paginated<Folder>, Error> {
        find(&self.db, None, profile_id, limit, cursor_raw, filters).await
    }

    async fn find_one_folder(&self, id: Uuid, profile_id: Uuid) -> Result<Folder, Error> {
        find_by_id(&self.db, id, profile_id).await
    }

    async fn create_folder(&self, data: FolderCreateData) -> Result<Folder, Error> {
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

    async fn update_folder(
        &self,
        id: Uuid,
        profile_id: Uuid,
        data: FolderUpdateData,
    ) -> Result<Folder, Error> {
        self.db
            .transaction::<_, Folder, Error>(|txn| {
                Box::pin(async move {
                    let Some(model) = queries::folder::select_by_id(txn, id, profile_id)
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

    async fn delete_folder(&self, id: Uuid, profile_id: Uuid) -> Result<(), Error> {
        let result = queries::folder::delete_by_id(&self.db, id, profile_id)
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

async fn find_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    profile_id: Uuid,
) -> Result<Folder, Error> {
    let folders = find(db, Some(id), profile_id, Some(1), None, None).await?;

    folders.data.first().cloned().ok_or(Error::NotFound(id))
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    pub title: String,
}
