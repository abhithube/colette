use async_trait::async_trait;
use chrono::DateTime;
use sea_orm::{
    ActiveModelTrait, ActiveValue, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    QueryFilter, TransactionTrait, prelude::Expr, sea_query::OnConflict,
};
use tower_sessions_core::{
    SessionStore,
    session::{Id, Record},
    session_store::{self, ExpiredDeletion},
};

use crate::repository::entity::sessions;

#[derive(Debug, Clone)]
pub struct SqliteStore {
    db: DatabaseConnection,
}

impl SqliteStore {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn try_create_with_conn(
        &self,
        tx: &DatabaseTransaction,
        record: &Record,
    ) -> session_store::Result<bool> {
        let data =
            serde_json::to_vec(record).map_err(|e| session_store::Error::Encode(e.to_string()))?;
        let expires_at = DateTime::from_timestamp(record.expiry_date.unix_timestamp(), 0)
            .unwrap()
            .to_rfc3339();

        let session = sessions::ActiveModel {
            id: ActiveValue::Set(record.id.to_string()),
            data: ActiveValue::Set(data),
            expires_at: ActiveValue::Set(expires_at),
        };

        match session.insert(tx).await {
            Ok(_) => Ok(true),
            Err(DbErr::RecordNotInserted) => Ok(false),
            Err(e) => Err(session_store::Error::Backend(e.to_string())),
        }
    }
}

#[async_trait]
impl SessionStore for SqliteStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        let tx = self
            .db
            .begin()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        while !self.try_create_with_conn(&tx, record).await? {
            record.id = Id::default();
        }

        tx.commit()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let data =
            serde_json::to_vec(record).map_err(|e| session_store::Error::Encode(e.to_string()))?;
        let expires_at = DateTime::from_timestamp(record.expiry_date.unix_timestamp(), 0)
            .unwrap()
            .to_rfc3339();

        let session = sessions::ActiveModel {
            id: ActiveValue::Set(record.id.to_string()),
            data: ActiveValue::Set(data),
            expires_at: ActiveValue::Set(expires_at),
        };

        sessions::Entity::insert(session)
            .on_conflict(
                OnConflict::columns([sessions::Column::Id])
                    .update_columns([sessions::Column::Data, sessions::Column::ExpiresAt])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let Some(session) = sessions::Entity::find_by_id(session_id.to_string())
            .filter(Expr::col(sessions::Column::ExpiresAt).gt(Expr::current_timestamp()))
            .one(&self.db)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?
        else {
            return Ok(None);
        };

        let data = serde_json::from_slice(&session.data)
            .map_err(|e| session_store::Error::Decode(e.to_string()))?;

        Ok(Some(data))
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        sessions::Entity::delete_by_id(session_id.to_string())
            .exec(&self.db)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl ExpiredDeletion for SqliteStore {
    async fn delete_expired(&self) -> session_store::Result<()> {
        sessions::Entity::delete_many()
            .filter(Expr::col(sessions::Column::ExpiresAt).lt(Expr::current_timestamp()))
            .exec(&self.db)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }
}
