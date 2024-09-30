use colette_entity::bookmark;
use sea_orm::{
    prelude::DateTimeWithTimeZone, sea_query::OnConflict, ConnectionTrait, DbErr, EntityTrait,
    InsertResult, Set,
};

pub async fn insert<Db: ConnectionTrait>(
    db: &Db,
    url: String,
    title: String,
    thumbnail_url: Option<String>,
    published_at: Option<DateTimeWithTimeZone>,
    author: Option<String>,
) -> Result<InsertResult<bookmark::ActiveModel>, DbErr> {
    let model = bookmark::ActiveModel {
        link: Set(url),
        title: Set(title),
        thumbnail_url: Set(thumbnail_url),
        published_at: Set(published_at),
        author: Set(author),
        ..Default::default()
    };

    bookmark::Entity::insert(model)
        .on_conflict(
            OnConflict::column(bookmark::Column::Link)
                .update_columns([
                    bookmark::Column::Title,
                    bookmark::Column::ThumbnailUrl,
                    bookmark::Column::PublishedAt,
                    bookmark::Column::Author,
                ])
                .to_owned(),
        )
        .exec(db)
        .await
}
