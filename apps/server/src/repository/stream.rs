use colette_core::{
    Stream,
    common::Transaction,
    stream::{
        Error, StreamById, StreamCreateData, StreamFindParams, StreamRepository, StreamUpdateData,
    },
};
use colette_model::streams;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteStreamRepository {
    db: DatabaseConnection,
}

impl SqliteStreamRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl StreamRepository for SqliteStreamRepository {
    async fn find_streams(&self, params: StreamFindParams) -> Result<Vec<Stream>, Error> {
        let streams = streams::Entity::find()
            .apply_if(params.id, |query, id| {
                query.filter(streams::Column::Id.eq(id.to_string()))
            })
            .apply_if(params.user_id, |query, user_id| {
                query.filter(streams::Column::UserId.eq(user_id.to_string()))
            })
            .apply_if(params.cursor, |query, cursor| {
                query.filter(streams::Column::Title.gt(cursor.title))
            })
            .order_by_asc(streams::Column::Title)
            .limit(params.limit.map(|e| e as u64))
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(streams)
    }

    async fn find_stream_by_id(&self, tx: &dyn Transaction, id: Uuid) -> Result<StreamById, Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let Some((id, user_id)) = streams::Entity::find()
            .select_only()
            .columns([streams::Column::Id, streams::Column::UserId])
            .filter(streams::Column::Id.eq(id.to_string()))
            .into_tuple::<(String, String)>()
            .one(tx)
            .await?
        else {
            return Err(Error::NotFound(id));
        };

        Ok(StreamById {
            id: id.parse().unwrap(),
            user_id: user_id.parse().unwrap(),
        })
    }

    async fn create_stream(&self, data: StreamCreateData) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();
        let model = streams::ActiveModel {
            id: ActiveValue::Set(id.into()),
            title: ActiveValue::Set(data.title.clone()),
            filter_raw: ActiveValue::Set(serde_json::to_string(&data.filter).unwrap()),
            user_id: ActiveValue::Set(data.user_id.into()),
            ..Default::default()
        };
        model.insert(&self.db).await.map_err(|e| match e {
            DbErr::RecordNotInserted => Error::Conflict(data.title),
            _ => Error::Database(e),
        })?;

        Ok(id)
    }

    async fn update_stream(
        &self,
        tx: &dyn Transaction,
        id: Uuid,
        data: StreamUpdateData,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let mut model = streams::ActiveModel {
            id: ActiveValue::Unchanged(id.to_string()),
            ..Default::default()
        };

        if let Some(title) = data.title {
            model.title = ActiveValue::Set(title);
        }
        if let Some(filter) = data.filter {
            model.filter_raw = ActiveValue::Set(serde_json::to_string(&filter).unwrap());
        }

        if model.is_changed() {
            model.update(tx).await?;
        }

        Ok(())
    }

    async fn delete_stream(&self, tx: &dyn Transaction, id: Uuid) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        streams::Entity::delete_by_id(id).exec(tx).await?;

        Ok(())
    }
}
