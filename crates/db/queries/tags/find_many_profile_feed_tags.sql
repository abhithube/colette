SELECT
  t.id,
  t.title,
  count(b.id) AS bookmark_count,
  count(pf.id) AS feed_count
FROM
  profile_feed_tags AS pft
  INNER JOIN tags AS t ON t.id = pft.tag_id
  LEFT JOIN bookmark_tags AS bt ON bt.tag_id = t.id
  LEFT JOIN bookmarks AS b ON b.id = bt.bookmark_id
  LEFT JOIN profile_feeds AS pf ON pf.id = pft.profile_feed_id
WHERE
  pft.profile_id = $1
GROUP BY
  t.id
ORDER BY
  t.title ASC;