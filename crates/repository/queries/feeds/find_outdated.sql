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
  refreshed_at IS NULL
  OR refreshed_at + (refresh_interval_min * INTERVAL '1 minute') <= now()
ORDER BY
  refreshed_at ASC,
  refresh_interval_min ASC
LIMIT
  $1
