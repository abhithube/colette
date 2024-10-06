use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    tag::{Cursor, Error, TagCreateData, TagFindManyFilters, TagRepository, TagUpdateData},
    Tag,
};
use sea_orm::{
    prelude::Uuid, sqlx, ActiveModelTrait, DatabaseConnection, IntoActiveModel, SqlErr,
    TransactionError, TransactionTrait,
};

use crate::query;

pub struct TagSqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl TagSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for TagSqlRepository {
    type Params = IdParams;
    type Output = Result<Tag, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        find_by_id(&self.db, params).await
    }
}

#[async_trait::async_trait]
impl Creatable for TagSqlRepository {
    type Data = TagCreateData;
    type Output = Result<Tag, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let model = query::tag::insert(
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

        find_by_id(&self.db, IdParams::new(model.id, data.profile_id)).await
    }
}

#[async_trait::async_trait]
impl Updatable for TagSqlRepository {
    type Params = IdParams;
    type Data = TagUpdateData;
    type Output = Result<Tag, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        self.db
            .transaction::<_, (), Error>(|txn| {
                Box::pin(async move {
                    let Some(model) = query::tag::select_by_id(txn, params.id, params.profile_id)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
                    };
                    let mut active_model = model.into_active_model();

                    if let Some(title) = data.title {
                        active_model.title.set_if_not_equals(title);
                    }
                    active_model.parent_id.set_if_not_equals(data.parent_id);

                    if active_model.is_changed() {
                        active_model
                            .update(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })?;

        find_by_id(&self.db, params).await
    }
}

#[async_trait::async_trait]
impl Deletable for TagSqlRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        colette_postgres::tag::delete_by_id(
            self.db.get_postgres_connection_pool(),
            params.id,
            params.profile_id,
        )
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }
}

#[async_trait::async_trait]
impl TagRepository for TagSqlRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
        filters: Option<TagFindManyFilters>,
    ) -> Result<Vec<Tag>, Error> {
        find(&self.db, None, profile_id, limit, cursor, filters).await
    }
}

pub(crate) async fn find(
    db: &DatabaseConnection,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<TagFindManyFilters>,
) -> Result<Vec<Tag>, Error> {
    let tags = colette_postgres::tag::select(
        db.get_postgres_connection_pool(),
        id,
        profile_id,
        limit,
        cursor,
        filters,
    )
    .await
    .map(|e| e.into_iter().map(Tag::from).collect::<Vec<_>>())
    .map_err(|e| Error::Unknown(e.into()))?;

    Ok(tags)
}

async fn find_by_id(db: &DatabaseConnection, params: IdParams) -> Result<Tag, Error> {
    let mut tags = find(db, Some(params.id), params.profile_id, None, None, None).await?;
    if tags.is_empty() {
        return Err(Error::NotFound(params.id));
    }

    Ok(tags.swap_remove(0))
}
