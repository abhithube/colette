use sqlx::PgExecutor;
use uuid::Uuid;

pub async fn select_by_url<'a>(ex: impl PgExecutor<'a>, url: String) -> sqlx::Result<Uuid> {
    sqlx::query_scalar!("SELECT id FROM feeds WHERE xml_url = $1 OR link = $1", url)
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
        "
INSERT INTO feeds (link, title, xml_url, updated_at)
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
