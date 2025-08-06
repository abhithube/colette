SELECT
  id,
  source_url AS "source_url: DbUrl",
  link AS "link: DbUrl",
  title,
  description,
  refresh_interval_min,
  status AS "status: DbFeedStatus",
  refreshed_at,
  is_custom
FROM
  feeds
WHERE
  (
    $1::UUID IS NULL
    OR id = $1
  )
  AND (
    $2::TEXT IS NULL
    OR source_url > $2
  )
ORDER BY
  title ASC
LIMIT
  $3
