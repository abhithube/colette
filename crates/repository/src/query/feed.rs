use colette_entity::{feed, profile_feed};
use futures::Stream;
use sea_orm::{
    sea_query::{Expr, Func, OnConflict, Query},
    ColumnTrait, ConnectionTrait, DbErr, DeleteResult, EntityTrait, InsertResult, QueryFilter,
    QuerySelect, Set, StreamTrait,
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

pub async fn delete_many<Db: ConnectionTrait>(db: &Db) -> Result<DeleteResult, DbErr> {
    let subquery = Query::select()
        .from(profile_feed::Entity)
        .and_where(
            Expr::col((profile_feed::Entity, profile_feed::Column::FeedId))
                .equals((feed::Entity, feed::Column::Id)),
        )
        .to_owned();

    feed::Entity::delete_many()
        .filter(Expr::exists(subquery).not())
        .exec(db)
        .await
}
