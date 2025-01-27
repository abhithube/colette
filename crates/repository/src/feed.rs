use std::fmt::Write;

use sea_query::Iden;
use sqlx::PgExecutor;
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

pub async fn select_by_url<'a>(ex: impl PgExecutor<'a>, url: String) -> sqlx::Result<Uuid> {
    sqlx::query_scalar!(
        "SELECT id
FROM feeds
WHERE xml_url = $1
OR link = $1",
        url
    )
    .fetch_one(ex)
    .await
}

pub async fn insert<'a>(
    ex: impl PgExecutor<'a>,
    link: String,
    title: String,
    xml_url: Option<String>,
) -> sqlx::Result<Uuid> {
    sqlx::query_scalar!(
        "INSERT INTO feeds (link, title, xml_url, updated_at)
VALUES ($1, $2, $3, now())
ON CONFLICT (link) DO UPDATE SET
    title = excluded.title,
    xml_url = excluded.xml_url,
    updated_at = excluded.updated_at
RETURNING id",
        link,
        title,
        xml_url
    )
    .fetch_one(ex)
    .await
}
