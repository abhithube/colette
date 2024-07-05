INSERT INTO
  profile_feed_entries (id, profile_feed_id, feed_entry_id)
VALUES
  ($1, $2, $3)
ON CONFLICT (profile_feed_id, feed_entry_id) DO nothing
RETURNING
  id;
