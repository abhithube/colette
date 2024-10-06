use colette_entity::feed_entry;
use sea_orm::{
    prelude::DateTimeWithTimeZone, sea_query::OnConflict, ColumnTrait, ConnectionTrait, DbErr,
    EntityTrait, QueryFilter, Set,
};

pub async fn select_many_by_feed_id<Db: ConnectionTrait>(
    db: &Db,
    feed_id: i32,
) -> Result<Vec<feed_entry::Model>, DbErr> {
    feed_entry::Entity::find()
        .filter(feed_entry::Column::FeedId.eq(feed_id))
        .all(db)
        .await
}

pub struct InsertMany {
    pub link: String,
    pub title: String,
    pub published_at: DateTimeWithTimeZone,
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
