WITH
  feed_count AS (
    SELECT
      uft.tag_id,
      coalesce(count(uft.user_feed_id), 0) AS count
    FROM
      user_feed_tags uft
    WHERE
      uft.user_id = $1
    GROUP BY
      uft.tag_id
  ),
  bookmark_count AS (
    SELECT
      bt.tag_id,
      coalesce(count(bt.bookmark_id), 0) AS count
    FROM
      bookmark_tags bt
    WHERE
      bt.user_id = $1
    GROUP BY
      bt.tag_id
  )
SELECT
  t.id,
  t.title,
  t.created_at,
  t.updated_at,
  fc.count AS feed_count,
  bc.count AS bookmark_count
FROM
  tags t
  LEFT JOIN feed_count fc ON fc.tag_id = t.id
  LEFT JOIN bookmark_count bc ON bc.tag_id = t.id
WHERE
  t.user_id = $1
  AND (
    $2::BOOLEAN
    OR t.id = $3
  )
  AND (
    $4::BOOLEAN
    OR t.title > $5
  )
ORDER BY
  t.title ASC
LIMIT
  $6
