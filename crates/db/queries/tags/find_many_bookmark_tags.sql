SELECT
  t.id,
  t.title,
  count(b.id) AS bookmark_count,
  count(pf.id) AS feed_count
FROM
  bookmark_tags AS bt
  INNER JOIN tags AS t ON t.id = bt.tag_id
  LEFT JOIN bookmarks AS b ON b.id = bt.bookmark_id
  LEFT JOIN profile_feed_tags AS pft ON pft.tag_id = t.id
  LEFT JOIN profile_feeds AS pf ON pf.id = pft.profile_feed_id
WHERE
  bt.profile_id = $1
GROUP BY
  t.id;