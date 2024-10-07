use colette_entity::feed;
use futures::Stream;
use sea_orm::{
    sea_query::{Expr, Func},
    ConnectionTrait, DbErr, EntityTrait, QuerySelect, StreamTrait,
};

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
