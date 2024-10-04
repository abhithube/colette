use colette_entity::{profile_bookmark_tag, profile_feed_tag, tag};
use sea_orm::{
    prelude::Uuid,
    sea_query::{Expr, Query},
    ColumnTrait, ConnectionTrait, DbErr, DeleteResult, EntityTrait, QueryFilter, Set,
};

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

pub async fn select_by_title_and_parent<Db: ConnectionTrait>(
    db: &Db,
    title: String,
    parent_id: Option<Uuid>,
    profile_id: Uuid,
) -> Result<Option<tag::Model>, DbErr> {
    tag::Entity::find()
        .filter(tag::Column::ProfileId.eq(profile_id))
        .filter(tag::Column::Title.eq(title))
        .filter(match parent_id {
            Some(parent_id) => tag::Column::ParentId.eq(parent_id),
            None => tag::Column::ParentId.is_null(),
        })
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

pub async fn delete_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    profile_id: Uuid,
) -> Result<DeleteResult, DbErr> {
    tag::Entity::delete_by_id(id)
        .filter(tag::Column::ProfileId.eq(profile_id))
        .exec(db)
        .await
}

pub async fn delete_many<Db: ConnectionTrait>(db: &Db) -> Result<DeleteResult, DbErr> {
    let feed_subquery = Query::select()
        .from(profile_feed_tag::Entity)
        .and_where(
            Expr::col((profile_feed_tag::Entity, profile_feed_tag::Column::TagId))
                .equals((tag::Entity, tag::Column::Id)),
        )
        .to_owned();

    let bookmark_subquery = Query::select()
        .from(profile_bookmark_tag::Entity)
        .and_where(
            Expr::col((
                profile_bookmark_tag::Entity,
                profile_bookmark_tag::Column::TagId,
            ))
            .equals((tag::Entity, tag::Column::Id)),
        )
        .to_owned();

    tag::Entity::delete_many()
        .filter(Expr::exists(feed_subquery).not())
        .filter(Expr::exists(bookmark_subquery).not())
        .exec(db)
        .await
}
