WITH
  b AS (
    INSERT INTO
      bookmarks (link, title, thumbnail_url, published_at, author)
    VALUES
      ($1, $2, $3, $4, $5)
    ON CONFLICT (link) DO
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
  ),
  pb_insert AS (
    INSERT INTO
      profile_bookmarks (profile_id, bookmark_id)
    SELECT
      $6,
      b.id
    FROM
      b
    ON CONFLICT (profile_id, bookmark_id) DO nothing
    RETURNING
      id,
      profile_id,
      bookmark_id
  ),
  pb AS (
    SELECT
      id AS "id!",
      profile_id,
      bookmark_id
    FROM
      pb_insert
    UNION ALL
    SELECT
      pb.id,
      pb.profile_id,
      pb.bookmark_id
    FROM
      profile_bookmarks pb,
      b
    WHERE
      pb.profile_id = $6
      AND pb.bookmark_id = b.id
  ),
  pbt AS (
    SELECT
      t.id,
      t.title,
      pbt.profile_bookmark_id
    FROM
      profile_bookmark_tags AS pbt
      INNER JOIN tags AS t ON t.id = pbt.tag_id
    ORDER BY
      t.title ASC
  )
SELECT
  pb."id!",
  b.link,
  b.title,
  b.thumbnail_url,
  b.published_at,
  b.author,
  coalesce(
    json_agg(
      DISTINCT jsonb_build_object(
        'id',
        pbt.id,
        'title',
        pbt.title,
        'bookmark_count',
        NULL::int8,
        'feed_count',
        NULL::int8
      )
    ) FILTER (
      WHERE
        pbt.id IS NOT NULL
    ),
    '[]'
  ) AS "tags!: Json<Vec<Tag>>"
FROM
  pb
  INNER JOIN b ON b.id = pb.bookmark_id
  LEFT JOIN pbt ON pbt.profile_bookmark_id = pb."id!"
GROUP BY
  pb."id!",
  b.link,
  b.title,
  b.thumbnail_url,
  b.published_at,
  b.author;