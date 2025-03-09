use std::fmt::Write;

use colette_core::user::{UserCreateParams, UserFindParams};
use sea_query::{Asterisk, Expr, Iden, InsertStatement, Query, SelectStatement};

use crate::{IntoInsert, IntoSelect};

pub enum User {
    Table,
    Id,
    Email,
    DisplayName,
    CreatedAt,
    UpdatedAt,
}

impl Iden for User {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "users",
                Self::Id => "id",
                Self::Email => "email",
                Self::DisplayName => "display_name",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

impl IntoSelect for UserFindParams {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column(Asterisk)
            .from(User::Table)
            .and_where(Expr::col(User::Id).eq(self.id.to_string()))
            .to_owned()
    }
}

impl IntoInsert for UserCreateParams {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(User::Table)
            .columns([User::Id, User::Email, User::DisplayName])
            .values_panic([
                self.id.to_string().into(),
                self.email.into(),
                self.display_name.into(),
            ])
            .to_owned()
    }
}
