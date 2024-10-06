use colette_entity::{profile, profile_feed};
use futures::Stream;
use sea_orm::{
    prelude::Uuid, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, JoinType, QueryFilter,
    QuerySelect, RelationTrait, StreamTrait,
};

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
