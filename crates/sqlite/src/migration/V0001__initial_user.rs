use colette_sql::{
    common::{WithPk, WithTimestamps},
    profile::Profile,
    user::User,
};
use sea_query::{
    ColumnDef, ConditionalStatement, Expr, ForeignKey, ForeignKeyAction, Iden, Index,
    SqliteQueryBuilder, Table,
};

pub fn migration() -> String {
    [
        Table::create()
            .table(User::Table)
            .if_not_exists()
            .with_uuid_pk()
            .col(ColumnDef::new(User::Email).text().not_null().unique_key())
            .col(ColumnDef::new(User::Password).text().not_null())
            .with_timestamps()
            .build(SqliteQueryBuilder),
        Table::create()
            .table(Profile::Table)
            .if_not_exists()
            .with_uuid_pk()
            .col(ColumnDef::new(Profile::Title).text().not_null())
            .col(ColumnDef::new(Profile::ImageUrl).text())
            .col(
                ColumnDef::new(Profile::IsDefault)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(ColumnDef::new(Profile::UserId).uuid().not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(Profile::Table, Profile::UserId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .with_timestamps()
            .build(SqliteQueryBuilder),
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
            .build(SqliteQueryBuilder),
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
            .build(SqliteQueryBuilder),
    ]
    .join("; ")
}
