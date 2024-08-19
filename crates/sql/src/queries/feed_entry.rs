use chrono::{DateTime, FixedOffset};
use colette_entities::{feed_entry, profile_feed_entry};
use sea_orm::{
    sea_query::{Expr, OnConflict, Query},
    ColumnTrait, ConnectionTrait, DbErr, DeleteResult, EntityTrait, QueryFilter, Set,
};

pub async fn select_many_in<Db: ConnectionTrait>(
    db: &Db,
    links: Vec<String>,
) -> Result<Vec<feed_entry::Model>, DbErr> {
    feed_entry::Entity::find()
        .filter(feed_entry::Column::Link.is_in(links))
        .all(db)
        .await
}

pub struct InsertMany {
    pub link: String,
    pub title: String,
    pub published_at: Option<DateTime<FixedOffset>>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<String>,
}

pub async fn insert_many<Db: ConnectionTrait>(
    db: &Db,
    fe: Vec<InsertMany>,
    feed_id: i32,
) -> Result<(), DbErr> {
    let models = fe
        .into_iter()
        .map(|e| feed_entry::ActiveModel {
            link: Set(e.link.to_string()),
            title: Set(e.title),
            published_at: Set(e.published_at),
            description: Set(e.description),
            author: Set(e.author),
            thumbnail_url: Set(e.thumbnail_url),
            feed_id: Set(feed_id),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    feed_entry::Entity::insert_many(models)
        .on_empty_do_nothing()
        .on_conflict(
            OnConflict::columns([feed_entry::Column::FeedId, feed_entry::Column::Link])
                .update_columns([
                    feed_entry::Column::Title,
                    feed_entry::Column::PublishedAt,
                    feed_entry::Column::Description,
                    feed_entry::Column::Author,
                    feed_entry::Column::ThumbnailUrl,
                ])
                .to_owned(),
        )
        .exec(db)
        .await?;

    Ok(())
}

pub async fn delete_many<Db: ConnectionTrait>(db: &Db) -> Result<DeleteResult, DbErr> {
    let subquery = Query::select()
        .from(profile_feed_entry::Entity)
        .and_where(
            Expr::col((
                profile_feed_entry::Entity,
                profile_feed_entry::Column::FeedEntryId,
            ))
            .equals((feed_entry::Entity, feed_entry::Column::Id)),
        )
        .to_owned();

    feed_entry::Entity::delete_many()
        .filter(Expr::exists(subquery).not())
        .exec(db)
        .await
}
