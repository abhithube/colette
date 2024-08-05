WITH
  pb AS (
    SELECT
      pb.id,
      b.link,
      b.title,
      b.thumbnail_url,
      b.published_at,
      b.author,
      pb.profile_id
    FROM
      profile_bookmarks AS pb
      INNER JOIN bookmarks AS b ON b.id = pb.bookmark_id
    WHERE
      pb.id = $1
      AND pb.profile_id = $2
  ),
  t AS (
    SELECT
      t.id,
      t.title
    FROM
      tags t,
      pb
    WHERE
      t.id = ANY ($3::UUID [])
      AND t.profile_id = pb.profile_id
  ),
  pbt_insert AS (
    INSERT INTO
      profile_bookmark_tags (profile_bookmark_id, tag_id, profile_id)
    SELECT
      pb.id,
      t.id,
      pb.profile_id
    FROM
      pb,
      t
    ON CONFLICT DO nothing
    RETURNING
      profile_bookmark_id,
      tag_id
  ),
  pbt_delete AS (
    DELETE FROM profile_bookmark_tags USING pb
    WHERE
      profile_bookmark_id = pb.id
      AND tag_id != ALL ($3::UUID [])
  ),
  pbt AS (
    SELECT
      profile_bookmark_id,
      tag_id
    FROM
      pbt_insert
    UNION ALL
    SELECT
      pbt.profile_bookmark_id,
      pbt.tag_id
    FROM
      profile_bookmark_tags pbt,
      pb
    WHERE
      pbt.profile_bookmark_id = pb.id
  )
SELECT
  pb.id,
  pb.link,
  pb.title,
  pb.thumbnail_url,
  pb.published_at,
  pb.author,
  coalesce(
    array_agg(ROW (t.id, t.title)) FILTER (
      WHERE
        t.id IS NOT NULL
    ),
    ARRAY[]::record[]
  ) AS "tags!: Vec<Tag>"
FROM
  pb
  LEFT JOIN pbt ON pbt.profile_bookmark_id = pb.id
  LEFT JOIN t ON pbt.tag_id = t.id
GROUP BY
  pb.id,
  pb.link,
  pb.title,
  pb.thumbnail_url,
  pb.published_at,
  pb.author;
