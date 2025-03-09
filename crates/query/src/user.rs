use colette_core::user::{UserCreateParams, UserFindParams};
use colette_model::users;
use sea_query::{Asterisk, Expr, InsertStatement, Query, SelectStatement};

use crate::{IntoInsert, IntoSelect};

impl IntoSelect for UserFindParams {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column(Asterisk)
            .from(users::Entity)
            .and_where(Expr::col(users::Column::Id).eq(self.id.to_string()))
            .to_owned()
    }
}

impl IntoInsert for UserCreateParams {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(users::Entity)
            .columns([
                users::Column::Id,
                users::Column::Email,
                users::Column::DisplayName,
            ])
            .values_panic([
                self.id.to_string().into(),
                self.email.into(),
                self.display_name.into(),
            ])
            .to_owned()
    }
}
