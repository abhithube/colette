use colette_entity::profile_feed;
use sea_orm::{
    prelude::Uuid, sea_query::OnConflict, ColumnTrait, ConnectionTrait, DbErr, EntityTrait,
    InsertResult, QueryFilter, Set,
};

pub async fn select_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    profile_id: Uuid,
) -> Result<Option<profile_feed::Model>, DbErr> {
    profile_feed::Entity::find_by_id(id)
        .filter(profile_feed::Column::ProfileId.eq(profile_id))
        .one(db)
        .await
}

pub async fn select_by_unique_index<Db: ConnectionTrait>(
    db: &Db,
    profile_id: Uuid,
    feed_id: i32,
) -> Result<Option<profile_feed::Model>, DbErr> {
    profile_feed::Entity::find()
        .filter(profile_feed::Column::ProfileId.eq(profile_id))
        .filter(profile_feed::Column::FeedId.eq(feed_id))
        .one(db)
        .await
}

pub async fn insert<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    pinned: Option<bool>,
    profile_id: Uuid,
    feed_id: i32,
) -> Result<InsertResult<profile_feed::ActiveModel>, DbErr> {
    let mut model = profile_feed::ActiveModel {
        id: Set(id),
        profile_id: Set(profile_id),
        feed_id: Set(feed_id),
        ..Default::default()
    };
    if let Some(pinned) = pinned {
        model.pinned = Set(pinned);
    }

    profile_feed::Entity::insert(model)
        .on_conflict(
            OnConflict::columns([
                profile_feed::Column::ProfileId,
                profile_feed::Column::FeedId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec(db)
        .await
}
