use colette_core::profile::Cursor;
use sea_query::{
    ColumnDef, ColumnType, ConditionalStatement, DeleteStatement, Expr, ForeignKey,
    ForeignKeyAction, Func, Iden, Index, IndexCreateStatement, InsertStatement, Order, Query,
    SelectStatement, Table, TableCreateStatement, UpdateStatement,
};
use uuid::Uuid;

use crate::{common::WithTimestamps, user::User};

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

pub fn create_table(id_type: ColumnType, timestamp_type: ColumnType) -> TableCreateStatement {
    Table::create()
        .table(Profile::Table)
        .if_not_exists()
        .col(
            ColumnDef::new_with_type(Profile::Id, id_type.clone())
                .not_null()
                .primary_key(),
        )
        .col(ColumnDef::new_with_type(Profile::Title, ColumnType::Text).not_null())
        .col(ColumnDef::new_with_type(
            Profile::ImageUrl,
            ColumnType::Text,
        ))
        .col(
            ColumnDef::new_with_type(Profile::IsDefault, ColumnType::Boolean)
                .not_null()
                .default(false),
        )
        .col(ColumnDef::new_with_type(Profile::UserId, id_type).not_null())
        .foreign_key(
            ForeignKey::create()
                .from(Profile::Table, Profile::UserId)
                .to(User::Table, User::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .with_timestamps(timestamp_type)
        .to_owned()
}

pub fn create_user_id_is_default_index() -> IndexCreateStatement {
    Index::create()
        .name(format!(
            "{profile}_{user_id}_{is_default}_idx",
            profile = Profile::Table.to_string(),
            user_id = Profile::UserId.to_string(),
            is_default = Profile::IsDefault.to_string()
        ))
        .table(Profile::Table)
        .if_not_exists()
        .col(Profile::UserId)
        .col(Profile::IsDefault)
        .unique()
        .and_where(Expr::col(Profile::IsDefault).into())
        .to_owned()
}

pub fn create_user_id_title_index() -> IndexCreateStatement {
    Index::create()
        .name(format!(
            "{profile}_{user_id}_{title}_idx",
            profile = Profile::Table.to_string(),
            user_id = Profile::UserId.to_string(),
            title = Profile::Title.to_string()
        ))
        .table(Profile::Table)
        .if_not_exists()
        .col(Profile::UserId)
        .col(Profile::Title)
        .unique()
        .to_owned()
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
