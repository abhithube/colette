use colette_core::feed::FeedFindManyFilters;
use colette_entity::{feed, profile_feed, profile_feed_tag, tag};
use sea_orm::{
    sea_query::{Expr, Func, OnConflict, SimpleExpr},
    ColumnTrait, Condition, ConnectionTrait, DbErr, DeleteResult, EntityTrait, InsertResult,
    JoinType, LoaderTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set,
};
use uuid::Uuid;

use crate::feed::Cursor;

pub async fn select_with_feed<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<FeedFindManyFilters>,
) -> Result<Vec<(profile_feed::Model, Option<feed::Model>)>, DbErr> {
    let mut query = profile_feed::Entity::find()
        .find_also_related(feed::Entity)
        .order_by_asc(SimpleExpr::FunctionCall(Func::coalesce([
            Expr::col((profile_feed::Entity, profile_feed::Column::Title)).into(),
            Expr::col((feed::Entity, feed::Column::Title)).into(),
        ])))
        .order_by_asc(profile_feed::Column::Id)
        .limit(limit);

    let mut conditions = Condition::all().add(profile_feed::Column::ProfileId.eq(profile_id));
    if let Some(id) = id {
        conditions = conditions.add(profile_feed::Column::Id.eq(id));
    }
    if let Some(filters) = filters {
        if let Some(tags) = filters.tags {
            query = query
                .join(
                    JoinType::InnerJoin,
                    profile_feed::Relation::ProfileFeedTag.def(),
                )
                .join(JoinType::InnerJoin, profile_feed_tag::Relation::Tag.def());

            conditions = conditions.add(tag::Column::Title.is_in(tags));
        }
    }
    if let Some(cursor) = cursor {
        conditions = conditions.add(
            Expr::tuple([
                Func::coalesce([
                    Expr::col((profile_feed::Entity, profile_feed::Column::Title)).into(),
                    Expr::col((feed::Entity, feed::Column::Title)).into(),
                ])
                .into(),
                Expr::col((profile_feed::Entity, profile_feed::Column::Id)).into(),
            ])
            .gt(Expr::tuple([
                Expr::value(cursor.title),
                Expr::value(cursor.id),
            ])),
        );
    }

    query.filter(conditions).all(db).await
}

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

pub async fn load_tags<Db: ConnectionTrait>(
    db: &Db,
    models: Vec<profile_feed::Model>,
) -> Result<Vec<Vec<tag::Model>>, DbErr> {
    models
        .load_many_to_many(
            tag::Entity::find().order_by_asc(tag::Column::Title),
            profile_feed_tag::Entity,
            db,
        )
        .await
}

pub async fn insert<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    profile_id: Uuid,
    feed_id: i32,
    folder_id: Option<Uuid>,
) -> Result<InsertResult<profile_feed::ActiveModel>, DbErr> {
    let model = profile_feed::ActiveModel {
        id: Set(id),
        profile_id: Set(profile_id),
        feed_id: Set(feed_id),
        folder_id: Set(folder_id),
        ..Default::default()
    };

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

pub async fn delete_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    profile_id: Uuid,
) -> Result<DeleteResult, DbErr> {
    profile_feed::Entity::delete_by_id(id)
        .filter(profile_feed::Column::ProfileId.eq(profile_id))
        .exec(db)
        .await
}
