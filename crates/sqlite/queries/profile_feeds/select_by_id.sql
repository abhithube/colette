SELECT
  pf.id,
  f.link,
  f.title,
  f.url,
  pf.custom_title,
  pf.created_at "created_at: chrono::DateTime<chrono::Utc>",
  pf.updated_at "updated_at: chrono::DateTime<chrono::Utc>",
  count(pfe.id) unread_count
FROM
  profile_feeds pf
  JOIN feeds f ON f.id = pf.feed_id
  JOIN feed_entries fe ON fe.feed_id = f.id
  LEFT JOIN profile_feed_entries pfe ON pfe.feed_entry_id = fe.id
  AND pfe.has_read = 0
WHERE
  pf.id = $1
  AND pf.profile_id = $2
GROUP BY
  pf.id,
  f.url,
  f.link,
  f.title;
