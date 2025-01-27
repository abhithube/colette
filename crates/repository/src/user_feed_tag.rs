use std::fmt::Write;

use sea_query::Iden;
use sqlx::PgExecutor;
use uuid::Uuid;

#[allow(dead_code)]
pub enum UserFeedTag {
    Table,
    UserFeedId,
    TagId,
    UserId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for UserFeedTag {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "user_feed_tags",
                Self::UserFeedId => "user_feed_id",
                Self::TagId => "tag_id",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

pub async fn insert_many<'a>(
    ex: impl PgExecutor<'a>,
    user_feed_id: Uuid,
    titles: &[String],
    user_id: Uuid,
) -> sqlx::Result<()> {
    sqlx::query_scalar!(
        "INSERT INTO user_feed_tags (user_feed_id, tag_id, user_id)
SELECT $1, id, user_id
FROM tags
WHERE user_id = $3
AND title = ANY($2)
ON CONFLICT (user_feed_id, tag_id) DO NOTHING",
        user_feed_id,
        titles,
        user_id
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn delete_many<'a>(
    ex: impl PgExecutor<'a>,
    titles: &[String],
    user_id: Uuid,
) -> sqlx::Result<()> {
    sqlx::query!(
        "DELETE FROM user_feed_tags
WHERE user_id = $1
AND tag_id IN (
    SELECT id
    FROM tags
    WHERE user_id = $1
    AND title = ANY($2)
)",
        user_id,
        titles
    )
    .execute(ex)
    .await?;

    Ok(())
}
