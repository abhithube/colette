use colette_entities::profile_bookmark_tag;
use sea_orm::{
    sea_query::OnConflict, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;

pub struct InsertMany {
    pub profile_bookmark_id: Uuid,
    pub tag_id: Uuid,
    pub profile_id: Uuid,
}

pub async fn insert_many<Db: ConnectionTrait>(db: &Db, pbt: Vec<InsertMany>) -> Result<(), DbErr> {
    let models = pbt
        .into_iter()
        .map(|e| profile_bookmark_tag::ActiveModel {
            profile_bookmark_id: Set(e.profile_bookmark_id),
            tag_id: Set(e.tag_id),
            profile_id: Set(e.profile_id),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    profile_bookmark_tag::Entity::insert_many(models)
        .on_empty_do_nothing()
        .on_conflict(
            OnConflict::columns([
                profile_bookmark_tag::Column::ProfileBookmarkId,
                profile_bookmark_tag::Column::TagId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec(db)
        .await?;

    Ok(())
}

pub async fn delete_many_not_in<Db: ConnectionTrait>(db: &Db, ids: Vec<Uuid>) -> Result<(), DbErr> {
    profile_bookmark_tag::Entity::delete_many()
        .filter(profile_bookmark_tag::Column::TagId.is_not_in(ids))
        .exec(db)
        .await?;

    Ok(())
}
