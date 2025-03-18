use std::fmt::Write;

use chrono::{DateTime, Utc};
use sea_query::{
    Asterisk, DeleteStatement, Expr, Iden, InsertStatement, Query, SelectStatement, UpdateStatement,
};

use crate::{IntoDelete, IntoInsert, IntoSelect, IntoUpdate};

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

pub enum UserSelectOne<'a> {
    Id(&'a str),
    Email(&'a str),
}

impl IntoSelect for UserSelectOne<'_> {
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

#[derive(Default)]
pub struct UserInsert<'a> {
    pub id: &'a str,
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
            .returning_all()
            .to_owned();

        query
    }
}

#[derive(Default)]
pub struct UserUpdate<'a> {
    pub id: &'a str,
    pub email: Option<&'a str>,
    pub verified_at: Option<Option<DateTime<Utc>>>,
    pub name: Option<Option<&'a str>>,
    pub password_hash: Option<Option<&'a str>>,
}

impl IntoUpdate for UserUpdate<'_> {
    fn into_update(self) -> UpdateStatement {
        let mut query = Query::update()
            .table(User::Table)
            .value(User::UpdatedAt, Expr::current_timestamp())
            .and_where(Expr::col(User::Id).eq(self.id))
            .to_owned();

        if let Some(email) = self.email {
            query.value(User::Email, email);
        }
        if let Some(verified_at) = self.verified_at {
            query.value(User::VerifiedAt, verified_at);
        }
        if let Some(name) = self.name {
            query.value(User::Name, name);
        }
        if let Some(password_hash) = self.password_hash {
            query.value(User::PasswordHash, password_hash);
        }

        query
    }
}

pub struct UserDelete<'a> {
    pub id: &'a str,
}

impl IntoDelete for UserDelete<'_> {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(User::Table)
            .and_where(Expr::col(User::Id).eq(self.id))
            .to_owned()
    }
}
