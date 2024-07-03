INSERT INTO
  profile_feed_entries (profile_feed_id, feed_entry_id)
VALUES
  (?1, ?2)
ON CONFLICT (profile_feed_id, feed_entry_id) DO NOTHING RETURNING id;
