SELECT coalesce(xml_url, link) AS "url!"
FROM feeds
JOIN user_feeds ON user_feeds.feed_id = feeds.id
