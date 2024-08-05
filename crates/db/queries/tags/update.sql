WITH
  t AS (
    UPDATE tags AS t
    SET
      title = coalesce($3, t.title)
    WHERE
      id = $1
      AND profile_id = $2
    RETURNING
      id,
      title
  )
SELECT
  t.id,
  t.title,
  count(b.id) AS bookmark_count,
  count(pf.id) AS feed_count
FROM
  t
  LEFT JOIN bookmark_tags AS bt ON bt.tag_id = t.id
  LEFT JOIN bookmarks AS b ON b.id = bt.bookmark_id
  LEFT JOIN profile_feed_tags AS pft ON pft.tag_id = t.id
  LEFT JOIN profile_feeds AS pf ON pf.id = pft.profile_feed_id
GROUP BY
  t.id,
  t.title;