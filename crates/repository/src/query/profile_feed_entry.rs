use colette_core::feed_entry::{Cursor, FeedEntryFindManyFilters};
use colette_entity::{
    feed_entry, profile_feed, profile_feed_entry, profile_feed_tag,
    sea_orm_active_enums::{Field, Operation},
    tag,
};
use sea_orm::{
    prelude::{DateTimeWithTimeZone, Uuid},
    sea_query::{Expr, OnConflict},
    ColumnTrait, Condition, ConnectionTrait, DatabaseBackend, DbErr, EntityTrait, JoinType,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set,
};

use super::smart_feed_filter;

pub async fn select_with_entry<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<FeedEntryFindManyFilters>,
) -> Result<Vec<(profile_feed_entry::Model, Option<feed_entry::Model>)>, DbErr> {
    let mut query = profile_feed_entry::Entity::find()
        .find_also_related(feed_entry::Entity)
        .order_by_desc(feed_entry::Column::PublishedAt)
        .order_by_desc(profile_feed_entry::Column::Id)
        .limit(limit);

    let mut conditions = Condition::all().add(profile_feed_entry::Column::ProfileId.eq(profile_id));
    if let Some(id) = id {
        conditions = conditions.add(profile_feed_entry::Column::Id.eq(id));
    }
    if let Some(filters) = filters {
        if let Some(feed_id) = filters.feed_id {
            conditions = conditions.add(profile_feed_entry::Column::ProfileFeedId.eq(feed_id));
        }
        if let Some(smart_feed_id) = filters.smart_feed_id {
            let filters =
                smart_feed_filter::select_by_smart_feed(db, smart_feed_id, profile_id).await?;

            for filter in filters {
                let column = match filter.field {
                    Field::Link => Expr::col((feed_entry::Entity, feed_entry::Column::Link)),
                    Field::Title => Expr::col((feed_entry::Entity, feed_entry::Column::Title)),
                    Field::PublishedAt => {
                        Expr::col((feed_entry::Entity, feed_entry::Column::PublishedAt))
                    }
                    Field::Description => {
                        Expr::col((feed_entry::Entity, feed_entry::Column::Description))
                    }
                    Field::Author => Expr::col((feed_entry::Entity, feed_entry::Column::Author)),
                    Field::HasRead => Expr::col((
                        profile_feed_entry::Entity,
                        profile_feed_entry::Column::HasRead,
                    )),
                };
                let expr = match filter.operation {
                    Operation::Equals => {
                        if let Ok(boolean) = filter.value.parse::<bool>() {
                            Some(column.eq(boolean))
                        } else {
                            Some(column.eq(filter.value))
                        }
                    }
                    Operation::Contains => Some(column.like(format!("%{}%", filter.value))),
                    Operation::GreaterThan => {
                        DateTimeWithTimeZone::parse_from_rfc3339(&filter.value)
                            .ok()
                            .map(|e| column.gt(e))
                    }
                    Operation::LessThan => DateTimeWithTimeZone::parse_from_rfc3339(&filter.value)
                        .ok()
                        .map(|e| column.lt(e)),
                    Operation::InLastMillis => filter.value.parse::<i64>().ok().and_then(|e| {
                        match db.get_database_backend() {
                            DatabaseBackend::Postgres => Some(
                                Expr::expr(Expr::cust_with_expr(
                                    "EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - $1))",
                                    column,
                                ))
                                .lt(e),
                            ),
                            DatabaseBackend::Sqlite => Some(
                                Expr::expr(Expr::cust_with_expr(
                                    "strftime('%s', CURRENT_TIMESTAMP) - strftime('%s', ?)",
                                    column,
                                ))
                                .lt(e),
                            ),
                            _ => None,
                        }
                    }),
                };

                if let Some(mut expr) = expr {
                    if filter.is_negated {
                        expr = expr.not();
                    }

                    conditions = conditions.add(expr);
                }
            }
        }
        if let Some(has_read) = filters.has_read {
            conditions = conditions.add(profile_feed_entry::Column::HasRead.eq(has_read));
        }
        if let Some(tags) = filters.tags {
            query = query
                .join(
                    JoinType::InnerJoin,
                    profile_feed_entry::Relation::ProfileFeed.def(),
                )
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
                Expr::col((feed_entry::Entity, feed_entry::Column::PublishedAt)).into(),
                Expr::col((profile_feed_entry::Entity, profile_feed_entry::Column::Id)).into(),
            ])
            .lt(Expr::tuple([
                Expr::value(cursor.published_at),
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
) -> Result<Option<profile_feed_entry::Model>, DbErr> {
    profile_feed_entry::Entity::find_by_id(id)
        .filter(profile_feed_entry::Column::ProfileId.eq(profile_id))
        .one(db)
        .await
}

pub async fn count_many_in_pfs<Db: ConnectionTrait>(
    db: &Db,
    ids: Vec<Uuid>,
) -> Result<Vec<(Uuid, i64)>, DbErr> {
    profile_feed_entry::Entity::find()
        .select_only()
        .column(profile_feed_entry::Column::ProfileFeedId)
        .column_as(profile_feed_entry::Column::Id.count(), "count")
        .filter(profile_feed_entry::Column::ProfileFeedId.is_in(ids))
        .filter(profile_feed_entry::Column::HasRead.eq(false))
        .group_by(profile_feed_entry::Column::ProfileFeedId)
        .into_tuple()
        .all(db)
        .await
}

pub struct InsertMany {
    pub id: Uuid,
    pub feed_entry_id: i32,
}

pub async fn insert_many<Db: ConnectionTrait>(
    db: &Db,
    pfe: Vec<InsertMany>,
    pf_id: Uuid,
    profile_id: Uuid,
) -> Result<(), DbErr> {
    let models = pfe
        .into_iter()
        .map(|e| profile_feed_entry::ActiveModel {
            id: Set(e.id),
            profile_feed_id: Set(pf_id),
            feed_entry_id: Set(e.feed_entry_id),
            profile_id: Set(profile_id),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    profile_feed_entry::Entity::insert_many(models)
        .on_empty_do_nothing()
        .on_conflict(
            OnConflict::columns([
                profile_feed_entry::Column::ProfileFeedId,
                profile_feed_entry::Column::FeedEntryId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec(db)
        .await?;

    Ok(())
}
