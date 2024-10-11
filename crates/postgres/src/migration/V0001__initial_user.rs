use colette_sql::{profile::Profile, user::User};
use sea_query::{
    ColumnDef, ColumnType, ConditionalStatement, Expr, ForeignKey, ForeignKeyAction, Iden, Index,
    PostgresQueryBuilder, Table,
};

use crate::migration::common::WithTimestamps;

pub fn migration() -> String {
    [
        Table::create()
            .table(User::Table)
            .if_not_exists()
            .col(
                ColumnDef::new_with_type(User::Id, ColumnType::Uuid)
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new_with_type(User::Email, ColumnType::Text)
                    .not_null()
                    .unique_key(),
            )
            .col(ColumnDef::new_with_type(User::Password, ColumnType::Text).not_null())
            .with_timestamps(ColumnType::TimestampWithTimeZone)
            .build(PostgresQueryBuilder),
        Table::create()
            .table(Profile::Table)
            .if_not_exists()
            .col(
                ColumnDef::new_with_type(Profile::Id, ColumnType::Uuid)
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
            .col(ColumnDef::new_with_type(Profile::UserId, ColumnType::Uuid).not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(Profile::Table, Profile::UserId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .with_timestamps(ColumnType::TimestampWithTimeZone)
            .build(PostgresQueryBuilder),
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
            .build(PostgresQueryBuilder),
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
            .build(PostgresQueryBuilder),
    ]
    .join("; ")
}
