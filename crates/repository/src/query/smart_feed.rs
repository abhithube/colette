use colette_entity::smart_feed;
use sea_orm::{
    prelude::Uuid, ColumnTrait, ConnectionTrait, DbErr, DeleteResult, EntityTrait, InsertResult,
    QueryFilter, Set,
};

pub async fn select_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    profile_id: Uuid,
) -> Result<Option<smart_feed::Model>, DbErr> {
    smart_feed::Entity::find_by_id(id)
        .filter(smart_feed::Column::ProfileId.eq(profile_id))
        .one(db)
        .await
}

pub async fn insert<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    title: String,
    profile_id: Uuid,
) -> Result<InsertResult<smart_feed::ActiveModel>, DbErr> {
    let model = smart_feed::ActiveModel {
        id: Set(id),
        title: Set(title),
        profile_id: Set(profile_id),
        ..Default::default()
    };

    smart_feed::Entity::insert(model).exec(db).await
}

pub async fn delete_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    profile_id: Uuid,
) -> Result<DeleteResult, DbErr> {
    smart_feed::Entity::delete_by_id(id)
        .filter(smart_feed::Column::ProfileId.eq(profile_id))
        .exec(db)
        .await
}
