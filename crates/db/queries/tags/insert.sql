INSERT INTO
  tag (id, title, profile_id)
VALUES
  ($1, $2, $3)
RETURNING
  id,
  title,
  0 AS "bookmark_count: i64",
  0 AS "feed_count: i64";