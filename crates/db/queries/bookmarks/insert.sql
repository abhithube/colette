WITH
  b AS (
    INSERT INTO
      bookmarks (
        link,
        title,
        thumbnail_url,
        published_at,
        author,
        profile_id
      )
    VALUES
      ($1, $2, $3, $4, $5, $6)
    ON CONFLICT (profile_id, link) DO
    UPDATE
    SET
      title = excluded.title,
      thumbnail_url = excluded.thumbnail_url,
      published_at = excluded.published_at,
      author = excluded.author
    RETURNING
      id,
      link,
      title,
      thumbnail_url,
      published_at,
      author
  )
SELECT
  b.id,
  b.link,
  b.title,
  b.thumbnail_url,
  b.published_at,
  b.author,
  coalesce(
    array_agg(ROW (t.id, t.title)) FILTER (
      WHERE
        t.id IS NOT NULL
    ),
    ARRAY[]::record[]
  ) AS "tags!: Vec<Tag>"
FROM
  b
  LEFT JOIN bookmark_tags AS bt ON bt.bookmark_id = b.id
  LEFT JOIN tags AS t ON bt.tag_id = t.id
GROUP BY
  b.id,
  b.link,
  b.title,
  b.thumbnail_url,
  b.published_at,
  b.author;