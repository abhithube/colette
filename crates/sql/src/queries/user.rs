use colette_entities::user;
use sea_orm::{ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

pub async fn select_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
) -> Result<Option<user::Model>, DbErr> {
    user::Entity::find_by_id(id).one(db).await
}

pub async fn select_by_email<Db: ConnectionTrait>(
    db: &Db,
    email: String,
) -> Result<Option<user::Model>, DbErr> {
    user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .one(db)
        .await
}

pub async fn insert<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    email: String,
    password: String,
) -> Result<user::Model, DbErr> {
    let model = user::ActiveModel {
        id: Set(id),
        email: Set(email),
        password: Set(password),
        ..Default::default()
    };

    user::Entity::insert(model).exec_with_returning(db).await
}
