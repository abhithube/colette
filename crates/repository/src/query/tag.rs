use colette_core::tag::{Cursor, TagFindManyFilters, TagType};
use colette_entity::{profile_bookmark_tag, profile_feed_tag, tag, PartialTag};
use sea_orm::{
    sea_query::{Alias, Expr, OnConflict},
    ColumnTrait, Condition, ConnectionTrait, DbErr, DeleteResult, EntityTrait, JoinType,
    QueryFilter, QuerySelect, RelationTrait, Set,
};
use uuid::Uuid;

pub async fn select<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<TagFindManyFilters>,
) -> Result<Vec<PartialTag>, DbErr> {
    let mut query = tag::Entity::find()
        .expr_as(
            Expr::col((
                Alias::new("pbt"),
                profile_bookmark_tag::Column::ProfileBookmarkId,
            ))
            .count(),
            "bookmark_count",
        )
        .expr_as(
            Expr::col((Alias::new("pft"), profile_feed_tag::Column::ProfileFeedId)).count(),
            "feed_count",
        )
        .join_as(
            JoinType::LeftJoin,
            tag::Relation::ProfileBookmarkTag.def(),
            Alias::new("pbt"),
        )
        .join_as(
            JoinType::LeftJoin,
            tag::Relation::ProfileFeedTag.def(),
            Alias::new("pft"),
        )
        .group_by(tag::Column::Id);

    let mut conditions = Condition::all().add(tag::Column::ProfileId.eq(profile_id));
    if let Some(id) = id {
        conditions = conditions.add(tag::Column::Id.eq(id));
    }
    if let Some(filters) = filters {
        query = match filters.tag_type {
            TagType::Bookmarks => {
                query.join(JoinType::InnerJoin, tag::Relation::ProfileBookmarkTag.def())
            }
            TagType::Feeds => query.join(JoinType::InnerJoin, tag::Relation::ProfileFeedTag.def()),
            _ => query,
        };
    }

    let mut query = query.filter(conditions).cursor_by(tag::Column::Title);
    if let Some(cursor) = cursor {
        query.after(cursor.title);
    }
    if let Some(limit) = limit {
        query.first(limit);
    }

    query.into_model::<PartialTag>().all(db).await
}

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

pub struct InsertMany {
    pub id: Uuid,
    pub title: String,
}

pub async fn insert_many<Db: ConnectionTrait>(
    db: &Db,
    tags: Vec<InsertMany>,
    profile_id: Uuid,
) -> Result<(), DbErr> {
    let models = tags
        .into_iter()
        .map(|e| tag::ActiveModel {
            id: Set(e.id),
            title: Set(e.title),
            profile_id: Set(profile_id),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    tag::Entity::insert_many(models)
        .on_empty_do_nothing()
        .on_conflict(
            OnConflict::columns([tag::Column::ProfileId, tag::Column::Title])
                .do_nothing()
                .to_owned(),
        )
        .exec(db)
        .await?;

    Ok(())
}

pub async fn select_by_tags<Db: ConnectionTrait>(
    db: &Db,
    tags: &[String],
) -> Result<Vec<tag::Model>, DbErr> {
    tag::Entity::find()
        .filter(tag::Column::Title.is_in(tags))
        .all(db)
        .await
}

pub async fn insert<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    title: String,
    profile_id: Uuid,
) -> Result<tag::Model, DbErr> {
    let model = tag::ActiveModel {
        id: Set(id),
        title: Set(title),
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
