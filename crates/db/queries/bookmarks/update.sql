WITH
  b AS (
    SELECT
      id,
      link,
      title,
      thumbnail_url,
      published_at,
      author,
      profile_id
    FROM
      bookmarks
    WHERE
      id = $1
      AND profile_id = $2
  ),
  t AS (
    SELECT
      t.id,
      t.title
    FROM
      tags t,
      b
    WHERE
      t.id = ANY ($3::UUID [])
      AND t.profile_id = b.profile_id
  ),
  bt_insert AS (
    INSERT INTO
      bookmark_tags (bookmark_id, tag_id, profile_id)
    SELECT
      b.id,
      t.id,
      b.profile_id
    FROM
      b,
      t
    ON CONFLICT DO nothing
    RETURNING
      bookmark_id,
      tag_id
  ),
  bt_delete AS (
    DELETE FROM bookmark_tags USING b
    WHERE
      bookmark_id = b.id
      AND tag_id != ALL ($3::UUID [])
  ),
  bt AS (
    SELECT
      bookmark_id,
      tag_id
    FROM
      bt_insert
    UNION ALL
    SELECT
      bt.bookmark_id,
      bt.tag_id
    FROM
      bookmark_tags bt,
      b
    WHERE
      bt.bookmark_id = b.id
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
  LEFT JOIN bt ON bt.bookmark_id = b.id
  LEFT JOIN t ON bt.tag_id = t.id
GROUP BY
  b.id,
  b.link,
  b.title,
  b.thumbnail_url,
  b.published_at,
  b.author;
