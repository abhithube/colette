use colette_core::folder::{FolderFindManyFilters, FolderType};
use colette_entity::{collection, folder, profile_feed, PartialFolder};
use sea_orm::{
    sea_query::{Alias, Expr},
    ColumnTrait, Condition, ConnectionTrait, DbErr, DeleteResult, EntityTrait, JoinType,
    QueryFilter, QuerySelect, RelationTrait, Set,
};
use uuid::Uuid;

use crate::folder::Cursor;

pub async fn select<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Cursor,
    filters: Option<FolderFindManyFilters>,
) -> Result<Vec<PartialFolder>, DbErr> {
    let mut query = folder::Entity::find()
        .expr_as(
            Expr::col((Alias::new("c"), collection::Column::FolderId)).count(),
            "collection_count",
        )
        .expr_as(
            Expr::col((Alias::new("pf"), profile_feed::Column::FolderId)).count(),
            "feed_count",
        )
        .join_as(
            JoinType::LeftJoin,
            folder::Relation::Collection.def(),
            Alias::new("c"),
        )
        .join_as(
            JoinType::LeftJoin,
            folder::Relation::ProfileFeed.def(),
            Alias::new("pf"),
        )
        .group_by(folder::Column::Id);

    let mut conditions = Condition::all().add(folder::Column::ProfileId.eq(profile_id));
    if let Some(id) = id {
        conditions = conditions.add(folder::Column::Id.eq(id));
    }
    if let Some(filters) = filters {
        query = match filters.folder_type {
            FolderType::Collections => {
                query.join(JoinType::InnerJoin, folder::Relation::Collection.def())
            }
            FolderType::Feeds => {
                query.join(JoinType::InnerJoin, folder::Relation::ProfileFeed.def())
            }
            _ => query,
        };
    }

    let mut query = query.filter(conditions).cursor_by(folder::Column::Title);
    query.after(cursor.title);
    if let Some(limit) = limit {
        query.first(limit);
    }

    query.into_model::<PartialFolder>().all(db).await
}

pub async fn select_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    profile_id: Uuid,
) -> Result<Option<folder::Model>, DbErr> {
    folder::Entity::find_by_id(id)
        .filter(folder::Column::ProfileId.eq(profile_id))
        .one(db)
        .await
}

pub async fn select_by_title_and_parent<Db: ConnectionTrait>(
    db: &Db,
    title: String,
    parent_id: Option<Uuid>,
    profile_id: Uuid,
) -> Result<Option<folder::Model>, DbErr> {
    folder::Entity::find()
        .filter(folder::Column::ProfileId.eq(profile_id))
        .filter(folder::Column::Title.eq(title))
        .filter(match parent_id {
            Some(parent_id) => folder::Column::ParentId.eq(parent_id),
            None => folder::Column::ParentId.is_null(),
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
) -> Result<folder::Model, DbErr> {
    let model = folder::ActiveModel {
        id: Set(id),
        title: Set(title),
        parent_id: Set(parent_id),
        profile_id: Set(profile_id),
        ..Default::default()
    };

    folder::Entity::insert(model).exec_with_returning(db).await
}

pub async fn delete_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    profile_id: Uuid,
) -> Result<DeleteResult, DbErr> {
    folder::Entity::delete_by_id(id)
        .filter(folder::Column::ProfileId.eq(profile_id))
        .exec(db)
        .await
}
