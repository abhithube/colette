use colette_entity::profile_bookmark;
use sea_orm::{
    prelude::Uuid,
    sea_query::{Expr, OnConflict, SimpleExpr},
    ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, InsertResult, QueryFilter,
    QueryOrder, Set,
};

pub async fn select_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    profile_id: Uuid,
) -> Result<Option<profile_bookmark::Model>, DbErr> {
    profile_bookmark::Entity::find_by_id(id)
        .filter(profile_bookmark::Column::ProfileId.eq(profile_id))
        .one(db)
        .await
}

pub async fn select_by_unique_index<Db: ConnectionTrait>(
    db: &Db,
    profile_id: Uuid,
    bookmark_id: i32,
) -> Result<Option<profile_bookmark::Model>, DbErr> {
    profile_bookmark::Entity::find()
        .filter(profile_bookmark::Column::ProfileId.eq(profile_id))
        .filter(profile_bookmark::Column::BookmarkId.eq(bookmark_id))
        .one(db)
        .await
}

pub async fn select_last<Db: ConnectionTrait>(
    db: &Db,
) -> Result<Option<profile_bookmark::Model>, DbErr> {
    profile_bookmark::Entity::find()
        .order_by_desc(profile_bookmark::Column::SortIndex)
        .one(db)
        .await
}

pub async fn insert<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    sort_index: i32,
    profile_id: Uuid,
    bookmark_id: i32,
) -> Result<InsertResult<profile_bookmark::ActiveModel>, DbErr> {
    let model = profile_bookmark::ActiveModel {
        id: Set(id),
        sort_index: Set(sort_index),
        profile_id: Set(profile_id),
        bookmark_id: Set(bookmark_id),
        ..Default::default()
    };

    profile_bookmark::Entity::insert(model)
        .on_conflict(
            OnConflict::columns([
                profile_bookmark::Column::ProfileId,
                profile_bookmark::Column::BookmarkId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec(db)
        .await
}

pub async fn update_many_sort_indexes<Db: ConnectionTrait>(
    db: &Db,
    sort_index: i32,
    old_sort_index: i32,
    profile_id: Uuid,
) -> Result<(), DbErr> {
    let mut conditions = Condition::all().add(profile_bookmark::Column::ProfileId.eq(profile_id));
    let expr: SimpleExpr;
    if sort_index > old_sort_index {
        conditions = conditions.add(
            profile_bookmark::Column::SortIndex
                .lte(sort_index)
                .and(profile_bookmark::Column::SortIndex.gt(old_sort_index)),
        );
        expr = Expr::col(profile_bookmark::Column::SortIndex).sub(1);
    } else {
        conditions = conditions.add(
            profile_bookmark::Column::SortIndex
                .gte(sort_index)
                .and(profile_bookmark::Column::SortIndex.lt(old_sort_index)),
        );
        expr = Expr::col(profile_bookmark::Column::SortIndex).add(1);
    }

    profile_bookmark::Entity::update_many()
        .col_expr(profile_bookmark::Column::SortIndex, expr)
        .filter(conditions)
        .exec(db)
        .await?;

    Ok(())
}

pub async fn decrement_many_sort_indexes<Db: ConnectionTrait>(
    db: &Db,
    sort_index: i32,
    profile_id: Uuid,
) -> Result<(), DbErr> {
    profile_bookmark::Entity::update_many()
        .col_expr(
            profile_bookmark::Column::SortIndex,
            Expr::col(profile_bookmark::Column::SortIndex).sub(1),
        )
        .filter(profile_bookmark::Column::ProfileId.eq(profile_id))
        .filter(profile_bookmark::Column::SortIndex.gt(sort_index))
        .exec(db)
        .await?;

    Ok(())
}
