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
  source_url = $1
