use colette_core::tag::{Cursor, TagFindManyFilters, TagType};
use colette_entity::{profile_bookmark_tag, profile_feed_tag, tag, PartialTag};
use sea_orm::{
    sea_query::{Alias, CommonTableExpression, Expr, OnConflict, Query, UnionType},
    ColumnTrait, ConnectionTrait, DbErr, DeleteResult, EntityTrait, FromQueryResult, JoinType,
    Order, QueryFilter, Set,
};
use uuid::Uuid;

pub(crate) fn tag_recursive_cte(profile_id: Uuid) -> CommonTableExpression {
    let tag_hierarchy = Alias::new("tag_hierarchy");
    let depth = Alias::new("depth");

    let mut base_query = Query::select()
        .column(tag::Column::Id)
        .column(tag::Column::Title)
        .column(tag::Column::ParentId)
        .expr_as(Expr::val(1), depth.clone())
        .from(tag::Entity)
        .and_where(tag::Column::ProfileId.eq(profile_id))
        .and_where(tag::Column::ParentId.is_null())
        .to_owned();

    let recursive_query = Query::select()
        .column((tag::Entity, tag::Column::Id))
        .column((tag::Entity, tag::Column::Title))
        .column((tag::Entity, tag::Column::ParentId))
        .expr(Expr::col((tag_hierarchy.clone(), depth)).add(1))
        .from(tag::Entity)
        .inner_join(
            tag_hierarchy.clone(),
            Expr::col((tag_hierarchy.clone(), tag::Column::Id))
                .eq(Expr::col((tag::Entity, tag::Column::ParentId))),
        )
        .to_owned();

    CommonTableExpression::new()
        .query(base_query.union(UnionType::All, recursive_query).to_owned())
        .table_name(tag_hierarchy)
        .to_owned()
}

pub async fn select<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<TagFindManyFilters>,
) -> Result<Vec<PartialTag>, DbErr> {
    let tag_hierarchy = Alias::new("tag_hierarchy");
    let tag_hierarchy2 = Alias::new("tag_hierarchy2");
    let depth = Alias::new("depth");

    let mut final_query = Query::select()
        .column((tag_hierarchy.clone(), tag::Column::Id))
        .column((tag_hierarchy.clone(), tag::Column::Title))
        .column((tag_hierarchy.clone(), tag::Column::ParentId))
        .column((tag_hierarchy.clone(), depth.clone()))
        .expr_as(
            profile_feed_tag::Column::ProfileFeedId.count(),
            Alias::new("feed_count"),
        )
        .expr_as(
            profile_bookmark_tag::Column::ProfileBookmarkId.count(),
            Alias::new("bookmark_count"),
        )
        .from(tag_hierarchy.clone())
        .join_as(
            JoinType::InnerJoin,
            tag_hierarchy.clone(),
            tag_hierarchy2.clone(),
            Expr::col((tag_hierarchy2.clone(), tag::Column::Id))
                .equals((tag_hierarchy.clone(), tag::Column::Id))
                .or(Expr::col((tag_hierarchy2.clone(), tag::Column::ParentId))
                    .equals((tag_hierarchy.clone(), tag::Column::Id))),
        )
        .left_join(
            profile_feed_tag::Entity,
            Expr::col((
                profile_feed_tag::Entity,
                profile_bookmark_tag::Column::TagId,
            ))
            .equals((tag_hierarchy2.clone(), tag::Column::Id)),
        )
        .left_join(
            profile_bookmark_tag::Entity,
            Expr::col((
                profile_bookmark_tag::Entity,
                profile_bookmark_tag::Column::TagId,
            ))
            .equals((tag_hierarchy2.clone(), tag::Column::Id)),
        )
        .and_where_option(id.map(|e| Expr::col((tag_hierarchy.clone(), tag::Column::Id)).eq(e)))
        .and_where_option(cursor.map(|e| tag::Column::Title.gt(e.title)))
        .group_by_columns([
            (tag_hierarchy.clone(), Alias::new("id")),
            (tag_hierarchy.clone(), Alias::new("title")),
            (tag_hierarchy.clone(), Alias::new("parent_id")),
            (tag_hierarchy.clone(), depth.clone()),
        ])
        .order_by((tag_hierarchy.clone(), depth), Order::Asc)
        .order_by((tag_hierarchy.clone(), tag::Column::Title), Order::Asc)
        .to_owned();

    if let Some(filters) = filters {
        match filters.tag_type {
            TagType::Bookmarks => {
                final_query.and_having(
                    Expr::expr(profile_bookmark_tag::Column::ProfileBookmarkId.count()).gt(0),
                );
            }
            TagType::Feeds => {
                final_query
                    .and_having(Expr::expr(profile_feed_tag::Column::ProfileFeedId.count()).gt(0));
            }
            _ => {}
        };

        final_query.and_where_option(
            filters
                .feed_id
                .map(|e| profile_feed_tag::Column::ProfileFeedId.eq(e)),
        );
        final_query.and_where_option(
            filters
                .bookmark_id
                .map(|e| profile_bookmark_tag::Column::ProfileBookmarkId.eq(e)),
        );
    }
    if let Some(limit) = limit {
        final_query.limit(limit);
    }

    let query = final_query.with(
        Query::with()
            .cte(tag_recursive_cte(profile_id))
            .recursive(true)
            .to_owned(),
    );

    PartialTag::find_by_statement(db.get_database_backend().build(&query))
        .all(db)
        .await
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

pub async fn prune_tag_list<Db: ConnectionTrait>(
    db: &Db,
    tag_ids: Vec<Uuid>,
    profile_id: Uuid,
) -> Result<Vec<Uuid>, DbErr> {
    let tag_hierarchy = Alias::new("tag_hierarchy");

    let mut base_query = Query::select()
        .column(tag::Column::Id)
        .column(tag::Column::ParentId)
        .from(tag::Entity)
        .and_where(tag::Column::ProfileId.eq(profile_id))
        .and_where(tag::Column::Id.is_in(tag_ids.clone()))
        .to_owned();

    let recursive_query = Query::select()
        .column((tag::Entity, tag::Column::Id))
        .column((tag::Entity, tag::Column::ParentId))
        .from(tag::Entity)
        .inner_join(
            tag_hierarchy.clone(),
            Expr::col((tag_hierarchy.clone(), tag::Column::Id))
                .eq(Expr::col((tag::Entity, tag::Column::ParentId))),
        )
        .to_owned();

    let subquery = Query::select()
        .expr(Expr::val(1))
        .from(tag_hierarchy.clone())
        .and_where(
            Expr::col((tag_hierarchy.clone(), tag::Column::ParentId))
                .eq(Expr::col((tag::Entity, tag::Column::Id))),
        )
        .and_where(Expr::col((tag_hierarchy.clone(), tag::Column::ParentId)).is_in(tag_ids.clone()))
        .to_owned();

    let final_query = Query::select()
        .distinct()
        .column(tag::Column::Id)
        .from(tag::Entity)
        .and_where(
            tag::Column::Id
                .is_in(tag_ids)
                .and(Expr::exists(subquery).not()),
        )
        .to_owned();

    let query = final_query.with(
        Query::with()
            .cte(
                CommonTableExpression::new()
                    .query(base_query.union(UnionType::All, recursive_query).to_owned())
                    .table_name(tag_hierarchy)
                    .to_owned(),
            )
            .recursive(true)
            .to_owned(),
    );

    let rows = db
        .query_all(db.get_database_backend().build(&query))
        .await?;

    let pruned = rows
        .into_iter()
        .filter_map(|e| e.try_get_by(0).ok())
        .collect::<Vec<Uuid>>();

    Ok(pruned)
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
