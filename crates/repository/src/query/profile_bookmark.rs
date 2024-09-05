use colette_core::bookmark::{BookmarkFindManyFilters, Cursor};
use colette_entity::{bookmark, profile_bookmark, profile_bookmark_tag, tag};
use sea_orm::{
    sea_query::{Expr, OnConflict, SimpleExpr},
    ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, InsertResult, JoinType,
    LoaderTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set,
};
use uuid::Uuid;

pub async fn select_with_bookmark<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<BookmarkFindManyFilters>,
) -> Result<Vec<(profile_bookmark::Model, Option<bookmark::Model>)>, DbErr> {
    let mut query = profile_bookmark::Entity::find()
        .find_also_related(bookmark::Entity)
        .order_by_asc(profile_bookmark::Column::SortIndex);

    let mut conditions = Condition::all().add(profile_bookmark::Column::ProfileId.eq(profile_id));
    if let Some(id) = id {
        conditions = conditions.add(profile_bookmark::Column::Id.eq(id));
    }
    if let Some(filters) = filters {
        if let Some(collection_id) = filters.collection_id {
            conditions = conditions.add(profile_bookmark::Column::CollectionId.eq(collection_id));
        }
        if let Some(tags) = filters.tags {
            query = query
                .join(
                    JoinType::InnerJoin,
                    profile_bookmark::Relation::ProfileBookmarkTag.def(),
                )
                .join(
                    JoinType::InnerJoin,
                    profile_bookmark_tag::Relation::Tag.def(),
                );

            conditions = conditions.add(tag::Column::Title.is_in(tags));
        }
    }

    let mut query = query
        .filter(conditions)
        .cursor_by(profile_bookmark::Column::SortIndex);
    if let Some(cursor) = cursor {
        query.after(cursor.sort_index);
    };
    if let Some(limit) = limit {
        query.first(limit);
    }

    query.all(db).await
}

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

pub async fn load_tags<Db: ConnectionTrait>(
    db: &Db,
    models: Vec<profile_bookmark::Model>,
) -> Result<Vec<Vec<tag::Model>>, DbErr> {
    models
        .load_many_to_many(
            tag::Entity::find().order_by_asc(tag::Column::Title),
            profile_bookmark_tag::Entity,
            db,
        )
        .await
}

pub async fn insert<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    sort_index: i32,
    profile_id: Uuid,
    bookmark_id: i32,
    collection_id: Option<Uuid>,
) -> Result<InsertResult<profile_bookmark::ActiveModel>, DbErr> {
    let model = profile_bookmark::ActiveModel {
        id: Set(id),
        sort_index: Set(sort_index),
        profile_id: Set(profile_id),
        bookmark_id: Set(bookmark_id),
        collection_id: Set(collection_id),
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
