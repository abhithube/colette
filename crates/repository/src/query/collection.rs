use colette_entity::{collection, profile_bookmark, PartialCollection};
use sea_orm::{
    sea_query::{Alias, Expr},
    ColumnTrait, Condition, ConnectionTrait, DbErr, DeleteResult, EntityTrait, JoinType,
    QueryFilter, QuerySelect, RelationTrait, Set,
};
use uuid::Uuid;

use crate::collection::Cursor;

pub async fn select<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Cursor,
) -> Result<Vec<PartialCollection>, DbErr> {
    let mut conditions = Condition::all().add(collection::Column::ProfileId.eq(profile_id));
    if let Some(id) = id {
        conditions = conditions.add(collection::Column::Id.eq(id));
    }

    let mut query = collection::Entity::find()
        .expr_as(
            Expr::col((Alias::new("pb"), profile_bookmark::Column::CollectionId)).count(),
            "bookmark_count",
        )
        .join_as(
            JoinType::LeftJoin,
            collection::Relation::ProfileBookmark.def(),
            Alias::new("pb"),
        )
        .group_by(collection::Column::Id)
        .filter(conditions)
        .cursor_by(collection::Column::Title);

    query.after(cursor.title);
    if let Some(limit) = limit {
        query.first(limit);
    }

    query.into_model::<PartialCollection>().all(db).await
}

pub async fn select_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    profile_id: Uuid,
) -> Result<Option<collection::Model>, DbErr> {
    collection::Entity::find_by_id(id)
        .filter(collection::Column::ProfileId.eq(profile_id))
        .one(db)
        .await
}

pub async fn insert<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    title: String,
    folder_id: Option<Uuid>,
    profile_id: Uuid,
) -> Result<collection::Model, DbErr> {
    let model = collection::ActiveModel {
        id: Set(id),
        title: Set(title),
        folder_id: Set(folder_id),
        profile_id: Set(profile_id),
        ..Default::default()
    };

    collection::Entity::insert(model)
        .exec_with_returning(db)
        .await
}

pub async fn delete_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    profile_id: Uuid,
) -> Result<DeleteResult, DbErr> {
    collection::Entity::delete_by_id(id)
        .filter(collection::Column::ProfileId.eq(profile_id))
        .exec(db)
        .await
}
