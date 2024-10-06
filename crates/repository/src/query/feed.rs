use colette_entity::feed;
use futures::Stream;
use sea_orm::{
    sea_query::{Expr, Func, OnConflict},
    ColumnTrait, ConnectionTrait, DbErr, EntityTrait, InsertResult, QueryFilter, QuerySelect, Set,
    StreamTrait,
};

pub async fn select_by_url<Db: ConnectionTrait>(
    db: &Db,
    url: String,
) -> Result<Option<feed::Model>, DbErr> {
    feed::Entity::find()
        .filter(
            feed::Column::Url
                .eq(url.clone())
                .or(feed::Column::Link.eq(url)),
        )
        .one(db)
        .await
}

pub async fn insert<Db: ConnectionTrait>(
    db: &Db,
    link: String,
    title: String,
    url: Option<String>,
) -> Result<InsertResult<feed::ActiveModel>, DbErr> {
    let model = feed::ActiveModel {
        link: Set(link),
        title: Set(title),
        url: Set(url),
        ..Default::default()
    };

    feed::Entity::insert(model)
        .on_conflict(
            OnConflict::column(feed::Column::Link)
                .update_columns([feed::Column::Title, feed::Column::Url])
                .to_owned(),
        )
        .exec(db)
        .await
}

pub async fn stream<Db: ConnectionTrait + StreamTrait>(
    db: &Db,
) -> Result<impl Stream<Item = Result<(i32, String), DbErr>> + Send + '_, DbErr> {
    feed::Entity::find()
        .select_only()
        .column(feed::Column::Id)
        .expr_as(
            Func::coalesce([
                Expr::col(feed::Column::Url).into(),
                Expr::col(feed::Column::Link).into(),
            ]),
            "url",
        )
        .into_tuple()
        .stream(db)
        .await
}
