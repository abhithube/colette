use colette_core::feed::{Cursor, FeedFindManyFilters};
use colette_entity::{feed, profile_feed, profile_feed_tag, tag, PartialFeedTag};
use indexmap::IndexMap;
use sea_orm::{
    sea_query::{
        Alias, CommonTableExpression, Expr, Func, OnConflict, Query, SimpleExpr, UnionType,
    },
    ColumnTrait, Condition, ConnectionTrait, DbErr, DeleteResult, EntityTrait, FromQueryResult,
    InsertResult, JoinType, Order, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set,
};
use uuid::Uuid;

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
        if let Some(pinned) = filters.pinned {
            query = query.filter(profile_feed::Column::Pinned.eq(pinned));
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
    pf_ids: Vec<Uuid>,
) -> Result<Vec<Vec<PartialFeedTag>>, DbErr> {
    let tag_tree = Alias::new("tag_tree");
    let level = Alias::new("level");

    let mut tag_map: IndexMap<Uuid, Vec<PartialFeedTag>> =
        IndexMap::from_iter(pf_ids.iter().map(|e| (*e, Vec::new())));

    let mut base_query = Query::select()
        .column(tag::Column::Id)
        .column(tag::Column::Title)
        .column(tag::Column::ParentId)
        .column(profile_feed_tag::Column::ProfileFeedId)
        .expr_as(Expr::val(1), level.clone())
        .from(tag::Entity)
        .inner_join(
            profile_feed_tag::Entity,
            Expr::col((profile_feed_tag::Entity, profile_feed_tag::Column::TagId))
                .equals((tag::Entity, tag::Column::Id)),
        )
        .and_where(profile_feed_tag::Column::ProfileFeedId.is_in(pf_ids))
        .to_owned();

    let recursive_query = Query::select()
        .column((tag::Entity, tag::Column::Id))
        .column((tag::Entity, tag::Column::Title))
        .column((tag::Entity, tag::Column::ParentId))
        .column((tag_tree.clone(), profile_feed_tag::Column::ProfileFeedId))
        .expr(Expr::col(level.clone()).add(1))
        .from(tag::Entity)
        .inner_join(
            tag_tree.clone(),
            Expr::col((tag_tree.clone(), tag::Column::ParentId))
                .equals((tag::Entity, tag::Column::Id)),
        )
        .to_owned();

    let final_query = Query::select()
        .column((tag_tree.clone(), tag::Column::Id))
        .column((tag_tree.clone(), tag::Column::Title))
        .column((tag_tree.clone(), tag::Column::ParentId))
        .column((tag_tree.clone(), profile_feed_tag::Column::ProfileFeedId))
        .column((tag_tree.clone(), level.clone()))
        .from(tag_tree.clone())
        .order_by((tag_tree.clone(), level), Order::Asc)
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

    let partial_tags = PartialFeedTag::find_by_statement(db.get_database_backend().build(&query))
        .all(db)
        .await?;

    for partial_tag in partial_tags {
        if let Some(tags) = tag_map.get_mut(&partial_tag.profile_feed_id) {
            tags.push(partial_tag);
        }
    }

    Ok(tag_map.into_values().collect::<Vec<_>>())
}

pub async fn insert<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    pinned: Option<bool>,
    profile_id: Uuid,
    feed_id: i32,
) -> Result<InsertResult<profile_feed::ActiveModel>, DbErr> {
    let mut model = profile_feed::ActiveModel {
        id: Set(id),
        profile_id: Set(profile_id),
        feed_id: Set(feed_id),
        ..Default::default()
    };
    if let Some(pinned) = pinned {
        model.pinned = Set(pinned);
    }

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
