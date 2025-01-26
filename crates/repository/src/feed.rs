use std::fmt::Write;

use sea_query::{Expr, Iden, InsertStatement, OnConflict, Query, SelectStatement, SimpleExpr};
use uuid::Uuid;

#[allow(dead_code)]
pub enum Feed {
    Table,
    Id,
    Link,
    Title,
    XmlUrl,
    CreatedAt,
    UpdatedAt,
}

impl Iden for Feed {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "feeds",
                Self::Id => "id",
                Self::Link => "link",
                Self::Title => "title",
                Self::XmlUrl => "xml_url",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

pub fn select_by_url(url: String) -> SelectStatement {
    Query::select()
        .column(Feed::Id)
        .from(Feed::Table)
        .and_where(
            Expr::col(Feed::XmlUrl)
                .eq(url.clone())
                .or(Expr::col(Feed::Link).eq(url)),
        )
        .to_owned()
}

pub fn insert(
    id: Option<Uuid>,
    link: String,
    title: String,
    xml_url: Option<String>,
) -> InsertStatement {
    let mut columns = vec![Feed::Link, Feed::Title, Feed::XmlUrl, Feed::UpdatedAt];
    let mut values: Vec<SimpleExpr> = vec![
        link.into(),
        title.into(),
        xml_url.into(),
        Expr::current_timestamp().into(),
    ];

    if let Some(id) = id {
        columns.push(Feed::Id);
        values.push(id.into());
    }

    Query::insert()
        .into_table(Feed::Table)
        .columns(columns)
        .values_panic(values)
        .on_conflict(
            OnConflict::column(Feed::Link)
                .update_columns([Feed::Title, Feed::XmlUrl, Feed::UpdatedAt])
                .to_owned(),
        )
        .returning_col(Feed::Id)
        .to_owned()
}
