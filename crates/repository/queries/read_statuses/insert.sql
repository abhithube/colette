INSERT INTO
  read_statuses (feed_entry_id, user_id, created_at, updated_at)
VALUES
  ($1, $2, $3, now())
