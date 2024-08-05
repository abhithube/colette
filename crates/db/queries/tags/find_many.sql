SELECT
  t.id,
  t.title,
  count(pb.id) AS bookmark_count,
  count(pf.id) AS feed_count
FROM
  tags AS t
  LEFT JOIN profile_bookmark_tags AS pbt ON pbt.tag_id = t.id
  LEFT JOIN profile_bookmarks AS pb ON pb.id = pbt.profile_bookmark_id
  LEFT JOIN profile_feed_tags AS pft ON pft.tag_id = t.id
  LEFT JOIN profile_feeds AS pf ON pf.id = pft.profile_feed_id
WHERE
  t.profile_id = $1
GROUP BY
  t.id
ORDER BY
  t.title ASC;