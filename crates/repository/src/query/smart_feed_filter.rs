use colette_entity::{
    sea_orm_active_enums::{Field, Operation},
    smart_feed_filter,
};
use sea_orm::{
    prelude::Uuid, ColumnTrait, ConnectionTrait, DbErr, DeleteResult, EntityTrait, InsertResult,
    QueryFilter, Set,
};

#[derive(Clone, Debug)]
pub struct InsertMany {
    pub id: Uuid,
    pub field: Field,
    pub operation: Operation,
    pub value: String,
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_many<Db: ConnectionTrait>(
    db: &Db,
    data: Vec<InsertMany>,
    smart_feed_id: Uuid,
    profile_id: Uuid,
) -> Result<InsertResult<smart_feed_filter::ActiveModel>, DbErr> {
    let models = data
        .into_iter()
        .map(|e| smart_feed_filter::ActiveModel {
            id: Set(e.id),
            field: Set(e.field),
            operation: Set(e.operation),
            value: Set(e.value),
            smart_feed_id: Set(smart_feed_id),
            profile_id: Set(profile_id),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    smart_feed_filter::Entity::insert_many(models)
        .exec(db)
        .await
}

pub async fn delete_many_by_smart_feed<Db: ConnectionTrait>(
    db: &Db,
    smart_feed_id: Uuid,
    profile_id: Uuid,
) -> Result<DeleteResult, DbErr> {
    smart_feed_filter::Entity::delete_many()
        .filter(smart_feed_filter::Column::SmartFeedId.eq(smart_feed_id))
        .filter(smart_feed_filter::Column::ProfileId.eq(profile_id))
        .exec(db)
        .await
}
