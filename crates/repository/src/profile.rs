use std::fmt::Write;

use colette_core::profile::Cursor;
use sea_query::{
    DeleteStatement, Expr, Func, Iden, InsertStatement, Order, Query, SelectStatement, SimpleExpr,
    UpdateStatement,
};
use uuid::Uuid;

#[allow(dead_code)]
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

impl Iden for Profile {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "profiles",
                Self::Id => "id",
                Self::Title => "title",
                Self::ImageUrl => "image_url",
                Self::IsDefault => "is_default",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
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
    id: Option<Uuid>,
    title: String,
    image_url: Option<String>,
    is_default: Option<bool>,
    user_id: Uuid,
) -> InsertStatement {
    let mut columns = vec![
        Profile::Title,
        Profile::ImageUrl,
        Profile::IsDefault,
        Profile::UserId,
    ];
    let mut values: Vec<SimpleExpr> = vec![
        title.into(),
        image_url.into(),
        is_default.unwrap_or_default().into(),
        user_id.into(),
    ];

    if let Some(id) = id {
        columns.push(Profile::Id);
        values.push(id.into());
    }

    Query::insert()
        .into_table(Profile::Table)
        .columns(columns)
        .values_panic(values)
        .returning_col(Profile::Id)
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
        .value(Profile::UpdatedAt, Expr::current_timestamp())
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
