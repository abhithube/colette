CREATE TRIGGER set_updated_at_users AFTER
UPDATE ON users FOR EACH ROW BEGIN
UPDATE users
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

CREATE TRIGGER set_updated_at_accounts AFTER
UPDATE ON accounts FOR EACH ROW BEGIN
UPDATE accounts
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

CREATE TRIGGER set_updated_at_api_keys AFTER
UPDATE ON api_keys FOR EACH ROW BEGIN
UPDATE api_keys
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

CREATE TRIGGER set_updated_at_feeds AFTER
UPDATE ON feeds FOR EACH ROW BEGIN
UPDATE feeds
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

CREATE TRIGGER set_updated_at_feed_entries AFTER
UPDATE ON feed_entries FOR EACH ROW BEGIN
UPDATE feed_entries
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

CREATE TRIGGER set_updated_at_user_feeds AFTER
UPDATE ON user_feeds FOR EACH ROW BEGIN
UPDATE user_feeds
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

CREATE TRIGGER set_updated_at_user_feed_entries AFTER
UPDATE ON user_feed_entries FOR EACH ROW BEGIN
UPDATE user_feed_entries
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

CREATE TRIGGER set_updated_at_bookmarks AFTER
UPDATE ON bookmarks FOR EACH ROW BEGIN
UPDATE bookmarks
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

CREATE TRIGGER set_updated_at_tags AFTER
UPDATE ON tags FOR EACH ROW BEGIN
UPDATE tags
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

CREATE TRIGGER set_updated_at_user_feed_tags AFTER
UPDATE ON user_feed_tags FOR EACH ROW BEGIN
UPDATE user_feed_tags
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

CREATE TRIGGER set_updated_at_bookmark_tags AFTER
UPDATE ON bookmark_tags FOR EACH ROW BEGIN
UPDATE bookmark_tags
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

CREATE TRIGGER set_updated_at_streams AFTER
UPDATE ON streams FOR EACH ROW BEGIN
UPDATE streams
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

CREATE TRIGGER set_updated_at_collections AFTER
UPDATE ON collections FOR EACH ROW BEGIN
UPDATE collections
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;
