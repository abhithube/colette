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
  last_refreshed_at IS NULL
  OR last_refreshed_at + (refresh_interval_min * INTERVAL '1 minute') <= now()
ORDER BY
  last_refreshed_at ASC,
  refresh_interval_min ASC
LIMIT
  $1
