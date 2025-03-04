use colette_core::{
    Collection,
    collection::{
        CollectionById, CollectionCreateData, CollectionFindParams, CollectionRepository,
        CollectionUpdateData, Error,
    },
    common::Transaction,
};
use colette_model::collections;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
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

    async fn find_collection_by_id(
        &self,
        tx: &dyn Transaction,
        id: Uuid,
    ) -> Result<CollectionById, Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let Some((id, user_id)) = collections::Entity::find()
            .select_only()
            .columns([collections::Column::Id, collections::Column::UserId])
            .filter(collections::Column::Id.eq(id.to_string()))
            .into_tuple::<(String, String)>()
            .one(tx)
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
        let model = collections::ActiveModel {
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

    async fn update_collection(
        &self,
        tx: &dyn Transaction,
        id: Uuid,
        data: CollectionUpdateData,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let mut model = collections::ActiveModel {
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

    async fn delete_collection(&self, tx: &dyn Transaction, id: Uuid) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        collections::Entity::delete_by_id(id).exec(tx).await?;

        Ok(())
    }
}
