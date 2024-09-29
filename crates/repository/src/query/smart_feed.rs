use colette_core::smart_feed::Cursor;
use colette_entity::{smart_feed, PartialSmartFeed};
use sea_orm::{
    prelude::Expr, ColumnTrait, Condition, ConnectionTrait, DbErr, DeleteResult, EntityTrait,
    InsertResult, QueryFilter, QueryOrder, QuerySelect, Set,
};
use uuid::Uuid;

pub async fn select<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
) -> Result<Vec<PartialSmartFeed>, DbErr> {
    let mut conditions = Condition::all().add(smart_feed::Column::ProfileId.eq(profile_id));
    if let Some(id) = id {
        conditions = conditions.add(smart_feed::Column::Id.eq(id));
    }

    let mut query = smart_feed::Entity::find()
        .expr_as(Expr::val(0), "unread_count")
        .filter(conditions)
        .cursor_by(smart_feed::Column::Title)
        .order_by_asc(smart_feed::Column::Title);

    if let Some(cursor) = cursor {
        query.after(cursor.title);
    }
    if let Some(limit) = limit {
        query.first(limit);
    }

    query.into_model::<PartialSmartFeed>().all(db).await
}

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
