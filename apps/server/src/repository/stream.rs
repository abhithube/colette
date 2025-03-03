use colette_core::{
    Stream,
    common::IdParams,
    stream::{
        Error, StreamById, StreamCreateData, StreamFindParams, StreamRepository, StreamUpdateData,
    },
};
use colette_model::streams;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    IntoActiveModel, ModelTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
    TransactionTrait,
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

    async fn find_stream_by_id(&self, id: Uuid) -> Result<StreamById, Error> {
        let Some((id, user_id)) = streams::Entity::find()
            .select_only()
            .columns([streams::Column::Id, streams::Column::UserId])
            .filter(streams::Column::Id.eq(id.to_string()))
            .into_tuple::<(String, String)>()
            .one(&self.db)
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
        let stream = streams::ActiveModel {
            id: ActiveValue::Set(id.into()),
            title: ActiveValue::Set(data.title.clone()),
            filter_raw: ActiveValue::Set(serde_json::to_string(&data.filter).unwrap()),
            user_id: ActiveValue::Set(data.user_id.into()),
            ..Default::default()
        };
        stream.insert(&self.db).await.map_err(|e| match e {
            DbErr::RecordNotInserted => Error::Conflict(data.title),
            _ => Error::Database(e),
        })?;

        Ok(id)
    }

    async fn update_stream(&self, params: IdParams, data: StreamUpdateData) -> Result<(), Error> {
        let tx = self.db.begin().await?;

        let Some(stream) = streams::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if stream.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        let mut stream = stream.into_active_model();

        if let Some(title) = data.title {
            stream.title = ActiveValue::Set(title);
        }
        if let Some(filter) = data.filter {
            stream.filter_raw = ActiveValue::Set(serde_json::to_string(&filter).unwrap());
        }

        if stream.is_changed() {
            stream.update(&tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn delete_stream(&self, params: IdParams) -> Result<(), Error> {
        let tx = self.db.begin().await?;

        let Some(stream) = streams::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if stream.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        stream.delete(&tx).await?;

        tx.commit().await?;

        Ok(())
    }
}
