SELECT
  pf.id,
  f.link,
  f.title,
  f.url,
  pf.custom_title,
  pf.created_at,
  pf.updated_at,
  count(pfe.id) unread_count
FROM
  profile_feeds pf
  JOIN feeds f ON f.id = pf.feed_id
  JOIN feed_entries fe ON fe.feed_id = f.id
  LEFT JOIN profile_feed_entries pfe ON pfe.feed_entry_id = fe.id
  AND pfe.has_read = FALSE
WHERE
  pf.profile_id = $1
GROUP BY
  pf.id,
  f.link,
  f.title,
  f.url
ORDER BY
  pf.custom_title ASC,
  f.title ASC;