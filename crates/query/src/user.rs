use std::fmt::Write;

use chrono::{DateTime, Utc};
use colette_core::user::UserParams;
use sea_query::{Asterisk, Expr, Iden, InsertStatement, OnConflict, Query, SelectStatement};
use uuid::Uuid;

use crate::{IntoInsert, IntoSelect};

pub enum User {
    Table,
    Id,
    Email,
    DisplayName,
    ImageUrl,
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
                Self::ImageUrl => "image_url",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

impl IntoSelect for UserParams {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column(Asterisk)
            .from(User::Table)
            .apply_if(self.id, |query, id| {
                query.and_where(Expr::col((User::Table, User::Id)).eq(id));
            })
            .to_owned()
    }
}

pub struct UserInsert<'a> {
    pub id: Uuid,
    pub email: &'a str,
    pub display_name: Option<&'a str>,
    pub image_url: Option<&'a str>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl IntoInsert for UserInsert<'_> {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(User::Table)
            .columns([
                User::Id,
                User::Email,
                User::DisplayName,
                User::ImageUrl,
                User::CreatedAt,
                User::UpdatedAt,
            ])
            .values_panic([
                self.id.into(),
                self.email.into(),
                self.display_name.into(),
                self.image_url.into(),
                self.created_at.into(),
                self.updated_at.into(),
            ])
            .on_conflict(
                OnConflict::column(User::Id)
                    .update_columns([
                        User::Email,
                        User::DisplayName,
                        User::ImageUrl,
                        User::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .to_owned()
    }
}
