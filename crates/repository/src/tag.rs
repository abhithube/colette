use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    tag::{Cursor, Error, TagCreateData, TagFindManyFilters, TagRepository, TagUpdateData},
    Tag,
};
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, IntoActiveModel, SqlErr,
    TransactionError, TransactionTrait,
};
use uuid::Uuid;

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
            data.profile_id,
        )
        .await
        .map_err(|e| match e.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => Error::Conflict(data.title),
            _ => Error::Unknown(e.into()),
        })?;

        Ok(Tag {
            id: model.id,
            title: model.title,
            bookmark_count: Some(0),
            feed_count: Some(0),
        })
    }
}

#[async_trait::async_trait]
impl Updatable for TagSqlRepository {
    type Params = IdParams;

    type Data = TagUpdateData;

    type Output = Result<Tag, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        self.db
            .transaction::<_, Tag, Error>(|txn| {
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
impl Deletable for TagSqlRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let result = query::tag::delete_by_id(&self.db, params.id, params.profile_id)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
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

async fn find<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<TagFindManyFilters>,
) -> Result<Vec<Tag>, Error> {
    let tags = query::tag::select(db, id, profile_id, limit.map(|e| e + 1), cursor, filters)
        .await
        .map(|e| e.into_iter().map(Tag::from).collect::<Vec<_>>())
        .map_err(|e| Error::Unknown(e.into()))?;

    Ok(tags)
}

async fn find_by_id<Db: ConnectionTrait>(db: &Db, params: IdParams) -> Result<Tag, Error> {
    let tags = find(db, Some(params.id), params.profile_id, None, None, None).await?;

    tags.first().cloned().ok_or(Error::NotFound(params.id))
}
