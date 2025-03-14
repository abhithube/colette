CREATE TABLE users (
  id TEXT NOT NULL PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  display_name TEXT,
  created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TRIGGER set_updated_at_users AFTER
UPDATE ON users FOR EACH ROW BEGIN
UPDATE users
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

CREATE TABLE accounts (
  provider_id TEXT NOT NULL,
  account_id TEXT NOT NULL,
  password_hash TEXT,
  user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (provider_id, account_id)
);

CREATE TRIGGER set_updated_at_accounts AFTER
UPDATE ON accounts FOR EACH ROW BEGIN
UPDATE accounts
SET
  updated_at = strftime('%s', 'now')
WHERE
  provider_id = OLD.provider_id
  AND account_id = OLD.account_id;

END;

CREATE TABLE api_keys (
  id TEXT NOT NULL PRIMARY KEY,
  lookup_hash TEXT NOT NULL UNIQUE,
  verification_hash TEXT NOT NULL,
  title TEXT NOT NULL,
  preview TEXT NOT NULL,
  user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TRIGGER set_updated_at_api_keys AFTER
UPDATE ON api_keys FOR EACH ROW BEGIN
UPDATE api_keys
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

CREATE TABLE feeds (
  id TEXT NOT NULL PRIMARY KEY,
  link TEXT NOT NULL UNIQUE,
  xml_url TEXT,
  title TEXT NOT NULL,
  description TEXT,
  refreshed_at INTEGER
);

CREATE TABLE feed_entries (
  id INTEGER NOT NULL PRIMARY KEY,
  link TEXT NOT NULL UNIQUE,
  title TEXT NOT NULL,
  published_at INTEGER NOT NULL,
  description TEXT,
  author TEXT,
  thumbnail_url TEXT,
  feed_id TEXT NOT NULL REFERENCES feeds (id) ON DELETE CASCADE,
  UNIQUE (feed_id, link)
);

CREATE TABLE subscriptions (
  id TEXT NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  feed_id TEXT NOT NULL REFERENCES feeds (id) ON DELETE RESTRICT,
  created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  UNIQUE (user_id, feed_id)
);

CREATE TRIGGER set_updated_at_subscriptions AFTER
UPDATE ON subscriptions FOR EACH ROW BEGIN
UPDATE subscriptions
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

CREATE TABLE subscription_entries (
  id TEXT NOT NULL PRIMARY KEY,
  has_read INTEGER NOT NULL DEFAULT 0,
  subscription_id TEXT NOT NULL REFERENCES subscriptions (id) ON DELETE CASCADE,
  feed_entry_id INTEGER NOT NULL REFERENCES feed_entries (id) ON DELETE RESTRICT,
  user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  UNIQUE (subscription_id, feed_entry_id)
);

CREATE TRIGGER set_updated_at_subscription_entries AFTER
UPDATE ON subscription_entries FOR EACH ROW BEGIN
UPDATE subscription_entries
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

CREATE TABLE bookmarks (
  id TEXT NOT NULL PRIMARY KEY,
  link TEXT NOT NULL,
  title TEXT NOT NULL,
  thumbnail_url TEXT,
  published_at INTEGER,
  author TEXT,
  archived_path TEXT,
  user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  UNIQUE (user_id, link)
);

CREATE TRIGGER set_updated_at_bookmarks AFTER
UPDATE ON bookmarks FOR EACH ROW BEGIN
UPDATE bookmarks
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

CREATE TABLE tags (
  id TEXT NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  UNIQUE (user_id, title)
);

CREATE TRIGGER set_updated_at_tags AFTER
UPDATE ON tags FOR EACH ROW BEGIN
UPDATE tags
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

CREATE TABLE subscription_tags (
  subscription_id TEXT NOT NULL REFERENCES subscriptions (id) ON DELETE CASCADE,
  tag_id TEXT NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
  user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (subscription_id, tag_id)
);

CREATE TRIGGER set_updated_at_subscription_tags AFTER
UPDATE ON subscription_tags FOR EACH ROW BEGIN
UPDATE subscription_tags
SET
  updated_at = strftime('%s', 'now')
WHERE
  subscription_id = OLD.subscription_id
  AND tag_id = OLD.tag_id;

END;

CREATE TABLE bookmark_tags (
  bookmark_id TEXT NOT NULL REFERENCES bookmarks (id) ON DELETE CASCADE,
  tag_id TEXT NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
  user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (bookmark_id, tag_id)
);

CREATE TRIGGER set_updated_at_bookmark_tags AFTER
UPDATE ON bookmark_tags FOR EACH ROW BEGIN
UPDATE bookmark_tags
SET
  updated_at = strftime('%s', 'now')
WHERE
  bookmark_id = OLD.bookmark_id
  AND tag_id = OLD.tag_id;

END;

CREATE TABLE streams (
  id TEXT NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT,
  filter_raw TEXT NOT NULL,
  user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  UNIQUE (user_id, title)
);

CREATE TRIGGER set_updated_at_streams AFTER
UPDATE ON streams FOR EACH ROW BEGIN
UPDATE streams
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

CREATE TABLE collections (
  id TEXT NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT,
  filter_raw TEXT NOT NULL,
  user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  UNIQUE (user_id, title)
);

CREATE TRIGGER set_updated_at_collections AFTER
UPDATE ON collections FOR EACH ROW BEGIN
UPDATE collections
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;
