use colette_core::tag::{Cursor, TagFindManyFilters, TagType};
use colette_entity::{profile_bookmark_tag, profile_feed_tag, tag, PartialTag};
use sea_orm::{
    sea_query::{Alias, CommonTableExpression, Expr, OnConflict, Query, UnionType},
    ColumnTrait, ConnectionTrait, DbErr, DeleteResult, EntityTrait, FromQueryResult, Order,
    QueryFilter, Set,
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
    let tag_tree = Alias::new("tag_tree");
    let root_id = Alias::new("root_id");

    let mut base_query = Query::select()
        .expr_as(Expr::col(tag::Column::Id), root_id.clone())
        .column(tag::Column::Title)
        .column(tag::Column::ParentId)
        .column(tag::Column::Id)
        .from(tag::Entity)
        .and_where(tag::Column::ProfileId.eq(profile_id))
        .and_where_option(id.map(|e| tag::Column::Id.eq(e)))
        .and_where_option(cursor.map(|e| tag::Column::Title.gt(e.title)))
        .to_owned();

    if let Some(filters) = filters {
        match filters.tag_type {
            TagType::Bookmarks => {
                base_query.inner_join(
                    profile_bookmark_tag::Entity,
                    Expr::col(profile_bookmark_tag::Column::TagId).equals(tag::Column::Id),
                );
            }
            TagType::Feeds => {
                base_query.inner_join(
                    profile_feed_tag::Entity,
                    Expr::col(profile_feed_tag::Column::TagId).equals(tag::Column::Id),
                );
            }
            _ => {}
        };
    }
    if let Some(limit) = limit {
        base_query.limit(limit);
    }

    let recursive_query = Query::select()
        .column((tag_tree.clone(), root_id.clone()))
        .column((tag_tree.clone(), tag::Column::Title))
        .column((tag_tree.clone(), tag::Column::ParentId))
        .column((tag::Entity, tag::Column::Id))
        .from(tag_tree.clone())
        .inner_join(
            tag::Entity,
            Expr::col((tag::Entity, tag::Column::ParentId))
                .equals((tag_tree.clone(), tag::Column::Id)),
        )
        .to_owned();

    let final_query = Query::select()
        .expr_as(
            Expr::col((tag_tree.clone(), root_id.clone())),
            Alias::new("id"),
        )
        .column((tag_tree.clone(), tag::Column::Title))
        .column((tag_tree.clone(), tag::Column::ParentId))
        .expr_as(
            profile_feed_tag::Column::ProfileFeedId.count(),
            Alias::new("feed_count"),
        )
        .expr_as(
            profile_bookmark_tag::Column::ProfileBookmarkId.count(),
            Alias::new("bookmark_count"),
        )
        .from(tag_tree.clone())
        .left_join(
            profile_feed_tag::Entity,
            Expr::col((profile_feed_tag::Entity, profile_feed_tag::Column::TagId))
                .equals((tag_tree.clone(), tag::Column::Id)),
        )
        .left_join(
            profile_bookmark_tag::Entity,
            Expr::col((
                profile_bookmark_tag::Entity,
                profile_bookmark_tag::Column::TagId,
            ))
            .equals((tag_tree.clone(), tag::Column::Id)),
        )
        .group_by_columns([
            (tag_tree.clone(), root_id),
            (tag_tree.clone(), Alias::new("title")),
            (tag_tree.clone(), Alias::new("parent_id")),
        ])
        .order_by((tag_tree.clone(), tag::Column::Title), Order::Asc)
        .to_owned();

    let query = final_query.with(
        Query::with()
            .cte(
                CommonTableExpression::new()
                    .query(base_query.union(UnionType::All, recursive_query).to_owned())
                    .table_name(tag_tree)
                    .to_owned(),
            )
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
