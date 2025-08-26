SELECT
  id,
  source_url AS "source_url: DbUrl",
  link AS "link: DbUrl",
  title,
  description,
  is_custom,
  status AS "status: DbFeedStatus",
  refresh_interval_min,
  last_refreshed_at,
  created_at,
  updated_at
FROM
  feeds
WHERE
  (
    $1::UUID IS NULL
    OR id = $1
  )
  AND (
    $2::TEXT IS NULL
    OR source_url = $2
  )
