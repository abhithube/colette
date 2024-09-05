use colette_core::profile::Cursor;
use colette_entity::{profile, profile_feed};
use futures::Stream;
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, JoinType, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, Set, StreamTrait,
};
use uuid::Uuid;

pub async fn select<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    user_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
) -> Result<Vec<profile::Model>, DbErr> {
    let query = profile::Entity::find().order_by_asc(profile::Column::Title);

    let mut conditions = Condition::all().add(profile::Column::UserId.eq(user_id));
    if let Some(id) = id {
        conditions = conditions.add(profile::Column::Id.eq(id));
    }

    let mut query = query.filter(conditions).cursor_by(profile::Column::Title);
    if let Some(cursor) = cursor {
        query.after(cursor.title);
    }
    if let Some(limit) = limit {
        query.first(limit);
    }

    query.all(db).await
}

pub async fn select_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    user_id: Uuid,
) -> Result<Option<profile::Model>, DbErr> {
    profile::Entity::find_by_id(id)
        .filter(profile::Column::UserId.eq(user_id))
        .one(db)
        .await
}

pub async fn select_default<Db: ConnectionTrait>(
    db: &Db,
    user_id: Uuid,
) -> Result<Option<profile::Model>, DbErr> {
    profile::Entity::find()
        .filter(profile::Column::UserId.eq(user_id))
        .filter(profile::Column::IsDefault.eq(true))
        .one(db)
        .await
}

pub async fn insert<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    title: String,
    image_url: Option<String>,
    is_default: Option<bool>,
    user_id: Uuid,
) -> Result<profile::Model, DbErr> {
    let mut model = profile::ActiveModel {
        id: Set(id),
        title: Set(title),
        image_url: Set(image_url),
        user_id: Set(user_id),
        ..Default::default()
    };
    if let Some(is_default) = is_default {
        model.is_default = Set(is_default);
    }

    profile::Entity::insert(model).exec_with_returning(db).await
}

pub async fn stream<Db: ConnectionTrait + StreamTrait>(
    db: &Db,
    feed_id: i32,
) -> Result<impl Stream<Item = Result<Uuid, DbErr>> + Send + '_, DbErr> {
    profile::Entity::find()
        .select_only()
        .column(profile::Column::Id)
        .join(JoinType::InnerJoin, profile::Relation::ProfileFeed.def())
        .filter(profile_feed::Column::FeedId.eq(feed_id))
        .into_tuple()
        .stream(db)
        .await
}
