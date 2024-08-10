SELECT
  t.id,
  t.title,
  t.slug,
  count(pb.id) AS bookmark_count,
  count(pf.id) AS feed_count
FROM
  tag AS t
  LEFT JOIN profile_bookmark_tag AS pbt ON pbt.tag_id = t.id
  LEFT JOIN profile_bookmark AS pb ON pb.id = pbt.profile_bookmark_id
  LEFT JOIN profile_feed_tag AS pft ON pft.tag_id = t.id
  LEFT JOIN profile_feed AS pf ON pf.id = pft.profile_feed_id
WHERE
  t.id = $1
  AND t.profile_id = $2
GROUP BY
  t.id;