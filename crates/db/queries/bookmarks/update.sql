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
  t_insert AS (
    INSERT INTO
      tags (title, profile_id)
    SELECT
      unnest($3::TEXT[]),
      pb.profile_id
    FROM
      pb
    ON CONFLICT (profile_id, title) DO nothing
    RETURNING
      id,
      title
  ),
  t AS (
    SELECT
      id,
      title
    FROM
      t_insert
    UNION ALL
    SELECT
      t.id,
      t.title
    FROM
      tags t,
      pb
    WHERE
      t.title = ANY ($3::TEXT[])
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
    DELETE FROM profile_bookmark_tags USING pb,
    t
    WHERE
      profile_bookmark_id = pb.id
      AND tag_id NOT IN (
        SELECT
          t.id
        FROM
          t
      )
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
    json_agg(
      DISTINCT jsonb_build_object(
        'id',
        t.id,
        'title',
        t.title,
        'bookmark_count',
        NULL::int8,
        'feed_count',
        NULL::int8
      )
    ) FILTER (
      WHERE
        t.id IS NOT NULL
    ),
    '[]'
  ) AS "tags!: Json<Vec<Tag>>"
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
