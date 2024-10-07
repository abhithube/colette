use colette_entity::profile_feed_entry;
use sea_orm::{prelude::Uuid, sea_query::OnConflict, ConnectionTrait, DbErr, EntityTrait, Set};

pub struct InsertMany {
    pub id: Uuid,
    pub feed_entry_id: i32,
}

pub async fn insert_many<Db: ConnectionTrait>(
    db: &Db,
    pfe: Vec<InsertMany>,
    pf_id: Uuid,
    profile_id: Uuid,
) -> Result<(), DbErr> {
    let models = pfe
        .into_iter()
        .map(|e| profile_feed_entry::ActiveModel {
            id: Set(e.id),
            profile_feed_id: Set(pf_id),
            feed_entry_id: Set(e.feed_entry_id),
            profile_id: Set(profile_id),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    profile_feed_entry::Entity::insert_many(models)
        .on_empty_do_nothing()
        .on_conflict(
            OnConflict::columns([
                profile_feed_entry::Column::ProfileFeedId,
                profile_feed_entry::Column::FeedEntryId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec(db)
        .await?;

    Ok(())
}
