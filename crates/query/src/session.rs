use std::fmt::Write;

use chrono::{DateTime, Utc};
use sea_query::{Asterisk, DeleteStatement, Expr, Iden, InsertStatement, Query, SelectStatement};

use crate::{IntoDelete, IntoInsert, IntoSelect};

pub enum Session {
    Table,
    Id,
    Token,
    UserAgent,
    IpAddress,
    ExpiresAt,
    UserId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for Session {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "sessions",
                Self::Id => "id",
                Self::Token => "token",
                Self::UserAgent => "user_agent",
                Self::IpAddress => "ip_address",
                Self::ExpiresAt => "expires_at",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

pub struct SessionSelect<'a> {
    pub token: &'a str,
}

impl IntoSelect for SessionSelect<'_> {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column(Asterisk)
            .from(Session::Table)
            .and_where(Expr::col(Session::Token).eq(self.token))
            .to_owned()
    }
}

pub struct SessionInsert<'a> {
    pub token: &'a str,
    pub user_agent: Option<&'a str>,
    pub ip_address: Option<&'a str>,
    pub expires_at: DateTime<Utc>,
    pub user_id: &'a str,
}

impl IntoInsert for SessionInsert<'_> {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(Session::Table)
            .columns([
                Session::Token,
                Session::UserAgent,
                Session::IpAddress,
                Session::ExpiresAt,
                Session::UserId,
            ])
            .values_panic([
                self.token.into(),
                self.user_agent.into(),
                self.ip_address.into(),
                self.expires_at.into(),
                self.user_id.into(),
            ])
            .returning_all()
            .to_owned()
    }
}

pub enum SessionDelete {
    Token(String),
    UserId(String),
    Expired,
}

impl IntoDelete for SessionDelete {
    fn into_delete(self) -> DeleteStatement {
        let r#where = match self {
            Self::Token(token) => Expr::col(Session::Token).eq(token),
            Self::UserId(user_id) => Expr::col(Session::UserId).eq(user_id),
            Self::Expired => Expr::col(Session::ExpiresAt).lt(Expr::current_timestamp()),
        };

        Query::delete()
            .from_table(Session::Table)
            .and_where(r#where)
            .to_owned()
    }
}
