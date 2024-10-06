use colette_entity::tag;
use sea_orm::{prelude::Uuid, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter, Set};

pub async fn select_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    profile_id: Uuid,
) -> Result<Option<tag::Model>, DbErr> {
    tag::Entity::find_by_id(id)
        .filter(tag::Column::ProfileId.eq(profile_id))
        .one(db)
        .await
}

pub async fn insert<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    title: String,
    parent_id: Option<Uuid>,
    profile_id: Uuid,
) -> Result<tag::Model, DbErr> {
    let model = tag::ActiveModel {
        id: Set(id),
        title: Set(title),
        parent_id: Set(parent_id),
        profile_id: Set(profile_id),
        ..Default::default()
    };

    tag::Entity::insert(model).exec_with_returning(db).await
}
