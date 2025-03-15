use std::fmt::Write;

use chrono::{DateTime, Utc};
use sea_query::{Asterisk, Expr, Iden, InsertStatement, Query, SelectStatement};
use uuid::Uuid;

use crate::{IntoInsert, IntoSelect};

pub enum User {
    Table,
    Id,
    Name,
    Email,
    VerifiedAt,
    PasswordHash,
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
                Self::Name => "name",
                Self::Email => "email",
                Self::VerifiedAt => "verified_at",
                Self::PasswordHash => "password_hash",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

pub enum UserSelectOne {
    Id(Uuid),
    Email(String),
}

impl IntoSelect for UserSelectOne {
    fn into_select(self) -> SelectStatement {
        let r#where = match self {
            Self::Id(id) => Expr::col(User::Id).eq(id),
            Self::Email(email) => Expr::col(User::Email).eq(email),
        };

        Query::select()
            .column(Asterisk)
            .from(User::Table)
            .and_where(r#where)
            .to_owned()
    }
}

pub struct UserInsert<'a> {
    pub id: Uuid,
    pub name: Option<&'a str>,
    pub email: &'a str,
    pub verified_at: Option<DateTime<Utc>>,
    pub password_hash: Option<&'a str>,
}

impl IntoInsert for UserInsert<'_> {
    fn into_insert(self) -> InsertStatement {
        let query = Query::insert()
            .into_table(User::Table)
            .columns([
                User::Id,
                User::Name,
                User::Email,
                User::VerifiedAt,
                User::PasswordHash,
            ])
            .values_panic([
                self.id.into(),
                self.name.into(),
                self.email.into(),
                self.verified_at.into(),
                self.password_hash.into(),
            ])
            .to_owned();

        query
    }
}
