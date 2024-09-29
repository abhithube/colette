use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    smart_feed::{Cursor, Error, SmartFeedCreateData, SmartFeedRepository, SmartFeedUpdateData},
    SmartFeed,
};
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, IntoActiveModel, SqlErr,
    TransactionError, TransactionTrait,
};
use uuid::Uuid;

use crate::query;

pub struct SmartFeedSqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl SmartFeedSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for SmartFeedSqlRepository {
    type Params = IdParams;
    type Output = Result<SmartFeed, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        find_by_id(&self.db, params).await
    }
}

#[async_trait::async_trait]
impl Creatable for SmartFeedSqlRepository {
    type Data = SmartFeedCreateData;
    type Output = Result<SmartFeed, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        self.db
            .transaction::<_, SmartFeed, Error>(|txn| {
                Box::pin(async move {
                    let model = query::smart_feed::insert(
                        txn,
                        Uuid::new_v4(),
                        data.title.clone(),
                        data.profile_id,
                    )
                    .await
                    .map_err(|e| match e.sql_err() {
                        Some(SqlErr::UniqueConstraintViolation(_)) => Error::Conflict(data.title),
                        _ => Error::Unknown(e.into()),
                    })?;

                    find_by_id(txn, IdParams::new(model.last_insert_id, data.profile_id)).await
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
impl Updatable for SmartFeedSqlRepository {
    type Params = IdParams;
    type Data = SmartFeedUpdateData;
    type Output = Result<SmartFeed, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        self.db
            .transaction::<_, SmartFeed, Error>(|txn| {
                Box::pin(async move {
                    let Some(model) =
                        query::smart_feed::select_by_id(txn, params.id, params.profile_id)
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
impl Deletable for SmartFeedSqlRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let result = query::smart_feed::delete_by_id(&self.db, params.id, params.profile_id)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl SmartFeedRepository for SmartFeedSqlRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
    ) -> Result<Vec<SmartFeed>, Error> {
        find(&self.db, None, profile_id, limit, cursor).await
    }
}

pub(crate) async fn find<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
) -> Result<Vec<SmartFeed>, Error> {
    let feeds = query::smart_feed::select(db, id, profile_id, limit, cursor)
        .await
        .map(|e| e.into_iter().map(SmartFeed::from).collect())
        .map_err(|e| Error::Unknown(e.into()))?;

    Ok(feeds)
}

async fn find_by_id<Db: ConnectionTrait>(db: &Db, params: IdParams) -> Result<SmartFeed, Error> {
    let mut feeds = find(db, Some(params.id), params.profile_id, None, None).await?;
    if feeds.is_empty() {
        return Err(Error::NotFound(params.id));
    }

    Ok(feeds.swap_remove(0))
}
