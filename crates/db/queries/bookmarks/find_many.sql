WITH
  pbt AS (
    SELECT
      t.id,
      t.title,
      pbt.profile_bookmark_id
    FROM
      profile_bookmark_tag AS pbt
      INNER JOIN tag AS t ON t.id = pbt.tag_id
    ORDER BY
      t.title ASC
  )
SELECT
  pb.id,
  b.link,
  b.title,
  b.thumbnail_url,
  b.published_at AS "published_at: DateTime<Utc>",
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
  profile_bookmark AS pb
  INNER JOIN bookmark AS b ON b.id = pb.bookmark_id
  LEFT JOIN pbt ON pbt.profile_bookmark_id = pb.id
WHERE
  pb.profile_id = $1
  AND (
    $3::UUID [] IS NULL
    OR pb.id IN (
      SELECT DISTINCT
        profile_bookmark_id
      FROM
        profile_bookmark_tag
      WHERE
        tag_id = ANY ($3)
    )
  )
GROUP BY
  pb.id,
  b.link,
  b.title,
  b.thumbnail_url,
  b.published_at,
  b.author
ORDER BY
  b.published_at DESC,
  b.title ASC,
  pb.id ASC
LIMIT
  $2;