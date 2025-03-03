use colette_core::{
    Collection,
    collection::{
        CollectionById, CollectionCreateData, CollectionFindParams, CollectionRepository,
        CollectionUpdateData, Error,
    },
    common::IdParams,
};
use colette_model::collections;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    IntoActiveModel, ModelTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
    TransactionTrait,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteCollectionRepository {
    db: DatabaseConnection,
}

impl SqliteCollectionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl CollectionRepository for SqliteCollectionRepository {
    async fn find_collections(
        &self,
        params: CollectionFindParams,
    ) -> Result<Vec<Collection>, Error> {
        let collections = collections::Entity::find()
            .apply_if(params.id, |query, id| {
                query.filter(collections::Column::Id.eq(id.to_string()))
            })
            .apply_if(params.user_id, |query, user_id| {
                query.filter(collections::Column::UserId.eq(user_id.to_string()))
            })
            .apply_if(params.cursor, |query, cursor| {
                query.filter(collections::Column::Title.gt(cursor.title))
            })
            .order_by_asc(collections::Column::Title)
            .limit(params.limit.map(|e| e as u64))
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(collections)
    }

    async fn find_collection_by_id(&self, id: Uuid) -> Result<CollectionById, Error> {
        let Some((id, user_id)) = collections::Entity::find()
            .select_only()
            .columns([collections::Column::Id, collections::Column::UserId])
            .filter(collections::Column::Id.eq(id.to_string()))
            .into_tuple::<(String, String)>()
            .one(&self.db)
            .await?
        else {
            return Err(Error::NotFound(id));
        };

        Ok(CollectionById {
            id: id.parse().unwrap(),
            user_id: user_id.parse().unwrap(),
        })
    }

    async fn create_collection(&self, data: CollectionCreateData) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();
        let collection = collections::ActiveModel {
            id: ActiveValue::Set(id.into()),
            title: ActiveValue::Set(data.title.clone()),
            filter_raw: ActiveValue::Set(serde_json::to_string(&data.filter).unwrap()),
            user_id: ActiveValue::Set(data.user_id.into()),
            ..Default::default()
        };
        collection.insert(&self.db).await.map_err(|e| match e {
            DbErr::RecordNotInserted => Error::Conflict(data.title),
            _ => Error::Database(e),
        })?;

        Ok(id)
    }

    async fn update_collection(
        &self,
        params: IdParams,
        data: CollectionUpdateData,
    ) -> Result<(), Error> {
        let tx = self.db.begin().await?;

        let Some(collection) = collections::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if collection.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        let mut collection = collection.into_active_model();

        if let Some(title) = data.title {
            collection.title = ActiveValue::Set(title);
        }
        if let Some(filter) = data.filter {
            collection.filter_raw = ActiveValue::Set(serde_json::to_string(&filter).unwrap());
        }

        if collection.is_changed() {
            collection.update(&tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn delete_collection(&self, params: IdParams) -> Result<(), Error> {
        let tx = self.db.begin().await?;

        let Some(collection) = collections::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if collection.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        collection.delete(&tx).await?;

        tx.commit().await?;

        Ok(())
    }
}
