use sea_query::{DeleteStatement, InsertStatement, SelectStatement, UpdateStatement};

pub mod api_key;
pub mod bookmark;
pub mod bookmark_tag;
pub mod collection;
pub mod feed;
pub mod feed_entry;
pub mod filter;
pub mod read_entry;
pub mod session;
pub mod stream;
pub mod subscription;
pub mod subscription_tag;
pub mod tag;
pub mod user;

pub trait IntoSelect {
    fn into_select(self) -> SelectStatement;
}

pub trait IntoInsert {
    fn into_insert(self) -> InsertStatement;
}

pub trait IntoUpdate {
    fn into_update(self) -> UpdateStatement;
}

pub trait IntoDelete {
    fn into_delete(self) -> DeleteStatement;
}
