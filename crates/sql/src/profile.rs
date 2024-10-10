use colette_core::profile::Cursor;
use sea_query::{
    DeleteStatement, Expr, Func, InsertStatement, Order, Query, SelectStatement, UpdateStatement,
};
use uuid::Uuid;

#[derive(sea_query::Iden)]
pub enum Profile {
    Table,
    Id,
    Title,
    ImageUrl,
    IsDefault,
    UserId,
    CreatedAt,
    UpdatedAt,
}

pub fn select(
    id: Option<Uuid>,
    user_id: Uuid,
    is_default: Option<bool>,
    cursor: Option<Cursor>,
    limit: Option<u64>,
) -> SelectStatement {
    let mut query = Query::select()
        .columns([
            Profile::Id,
            Profile::Title,
            Profile::ImageUrl,
            Profile::IsDefault,
            Profile::UserId,
        ])
        .from(Profile::Table)
        .and_where(Expr::col((Profile::Table, Profile::UserId)).eq(user_id))
        .and_where_option(id.map(|e| Expr::col((Profile::Table, Profile::Id)).eq(e)))
        .and_where_option(is_default.map(|e| Expr::col((Profile::Table, Profile::IsDefault)).eq(e)))
        .and_where_option(cursor.map(|e| Expr::col((Profile::Table, Profile::Title)).gt(e.title)))
        .order_by((Profile::Table, Profile::Title), Order::Asc)
        .to_owned();

    if let Some(limit) = limit {
        query.limit(limit);
    }

    query
}

pub fn insert(
    id: Uuid,
    title: String,
    image_url: Option<String>,
    is_default: Option<bool>,
    user_id: Uuid,
) -> InsertStatement {
    Query::insert()
        .into_table(Profile::Table)
        .columns([
            Profile::Id,
            Profile::Title,
            Profile::ImageUrl,
            Profile::IsDefault,
            Profile::UserId,
        ])
        .values_panic([
            id.into(),
            title.into(),
            image_url.into(),
            is_default.unwrap_or_default().into(),
            user_id.into(),
        ])
        .to_owned()
}

pub fn update(
    id: Uuid,
    user_id: Uuid,
    title: Option<String>,
    image_url: Option<Option<String>>,
) -> UpdateStatement {
    let mut query = Query::update()
        .table(Profile::Table)
        .and_where(Expr::col((Profile::Table, Profile::Id)).eq(id))
        .and_where(Expr::col((Profile::Table, Profile::UserId)).eq(user_id))
        .to_owned();

    if let Some(title) = title {
        query.value(
            Profile::Title,
            Func::coalesce([
                title.into(),
                Expr::col((Profile::Table, Profile::Title)).into(),
            ]),
        );
    }
    if let Some(image_url) = image_url {
        query.value(
            Profile::ImageUrl,
            Func::coalesce([
                image_url.into(),
                Expr::col((Profile::Table, Profile::ImageUrl)).into(),
            ]),
        );
    }

    query
}

pub fn delete(id: Uuid, user_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(Profile::Table)
        .and_where(Expr::col((Profile::Table, Profile::Id)).eq(id))
        .and_where(Expr::col((Profile::Table, Profile::UserId)).eq(user_id))
        .to_owned()
}
