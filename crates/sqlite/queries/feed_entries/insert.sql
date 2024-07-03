INSERT INTO
  feed_entries (feed_id, entry_id)
VALUES
  (?1, ?2)
ON CONFLICT (feed_id, entry_id) DO NOTHING RETURNING id;