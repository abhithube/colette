use colette_entity::profile_feed_tag;
use sea_orm::{
    sea_query::OnConflict, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;

pub struct InsertMany {
    pub profile_feed_id: Uuid,
    pub tag_id: Uuid,
}

pub async fn insert_many<Db: ConnectionTrait>(
    db: &Db,
    pft: Vec<InsertMany>,
    profile_id: Uuid,
) -> Result<(), DbErr> {
    let models = pft
        .into_iter()
        .map(|e| profile_feed_tag::ActiveModel {
            profile_feed_id: Set(e.profile_feed_id),
            tag_id: Set(e.tag_id),
            profile_id: Set(profile_id),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    profile_feed_tag::Entity::insert_many(models)
        .on_empty_do_nothing()
        .on_conflict(
            OnConflict::columns([
                profile_feed_tag::Column::ProfileFeedId,
                profile_feed_tag::Column::TagId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec(db)
        .await?;

    Ok(())
}

pub async fn delete_many_in<Db: ConnectionTrait>(
    db: &Db,
    profile_feed_id: Uuid,
    ids: Vec<Uuid>,
) -> Result<(), DbErr> {
    profile_feed_tag::Entity::delete_many()
        .filter(profile_feed_tag::Column::ProfileFeedId.eq(profile_feed_id))
        .filter(profile_feed_tag::Column::TagId.is_in(ids))
        .exec(db)
        .await?;

    Ok(())
}

pub async fn delete_many_not_in<Db: ConnectionTrait>(
    db: &Db,
    profile_feed_id: Uuid,
    ids: Vec<Uuid>,
) -> Result<(), DbErr> {
    profile_feed_tag::Entity::delete_many()
        .filter(profile_feed_tag::Column::ProfileFeedId.eq(profile_feed_id))
        .filter(profile_feed_tag::Column::TagId.is_not_in(ids))
        .exec(db)
        .await?;

    Ok(())
}
