use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

pub async fn select_many_by_feed_id<'a>(
    ex: impl PgExecutor<'a>,
    feed_id: Uuid,
) -> sqlx::Result<Vec<Uuid>> {
    sqlx::query_scalar!("SELECT id FROM feed_entries WHERE feed_id = $1", feed_id)
        .fetch_all(ex)
        .await
}

pub struct InsertMany {
    pub link: String,
    pub title: String,
    pub published_at: DateTime<Utc>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<String>,
}

pub async fn insert_many<'a>(
    ex: impl PgExecutor<'a>,
    data: Vec<InsertMany>,
    feed_id: Uuid,
) -> sqlx::Result<()> {
    let mut links = Vec::<String>::new();
    let mut titles = Vec::<String>::new();
    let mut published_ats = Vec::<NaiveDateTime>::new();
    let mut descriptions = Vec::<Option<String>>::new();
    let mut authors = Vec::<Option<String>>::new();
    let mut thumbnail_urls = Vec::<Option<String>>::new();

    for item in data {
        links.push(item.link);
        titles.push(item.title);
        published_ats.push(item.published_at.naive_utc());
        descriptions.push(item.description);
        authors.push(item.author);
        thumbnail_urls.push(item.thumbnail_url);
    }

    sqlx::query_scalar!(
        "
INSERT INTO feed_entries (link, title, published_at, description, author, thumbnail_url, feed_id, updated_at)
SELECT *, $7::uuid, now()
FROM UNNEST($1::text[], $2::text[], $3::timestamp[], $4::text[], $5::text[], $6::text[])
ON CONFLICT (feed_id, link) DO UPDATE SET
    title = excluded.title,
    published_at = excluded.published_at,
    description = excluded.description,
    author = excluded.author,
    thumbnail_url = excluded.thumbnail_url,
    updated_at = excluded.updated_at",
        &links,
        &titles,
        &published_ats,
        &descriptions as &[Option<String>],
        &authors as &[Option<String>],
        &thumbnail_urls as &[Option<String>],
        feed_id
    )
    .execute(ex)
    .await?;

    Ok(())
}
