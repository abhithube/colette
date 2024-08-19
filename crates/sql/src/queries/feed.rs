use colette_entities::{feed, profile_feed};
use futures::Stream;
use sea_orm::{
    sea_query::{Expr, Func, OnConflict, Query},
    ConnectionTrait, DbErr, DeleteResult, EntityTrait, InsertResult, QueryFilter, QuerySelect, Set,
    StreamTrait,
};

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

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
pub struct StreamSelect {
    pub id: i32,
    pub url: String,
}

pub async fn stream<Db: ConnectionTrait + StreamTrait>(
    db: &Db,
) -> Result<impl Stream<Item = Result<StreamSelect, DbErr>> + Send + '_, DbErr> {
    feed::Entity::find()
        .expr_as(
            Func::coalesce([
                Expr::col(feed::Column::Url).into(),
                Expr::col(feed::Column::Link).into(),
            ]),
            "url",
        )
        .into_model::<StreamSelect>()
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
