-- Disable the enforcement of foreign-keys constraints
PRAGMA foreign_keys = off;

-- Drop "user_feeds" table
DROP TABLE `user_feeds`;

-- Drop "user_feed_entries" table
DROP TABLE `user_feed_entries`;

-- Drop "user_feed_tags" table
DROP TABLE `user_feed_tags`;

-- Create "subscriptions" table
CREATE TABLE `subscriptions` (
  `id` TEXT NOT NULL,
  `title` TEXT NOT NULL,
  `user_id` TEXT NOT NULL,
  `feed_id` INTEGER NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`feed_id`) REFERENCES `feeds` (`id`) ON UPDATE NO ACTION ON DELETE RESTRICT,
  CONSTRAINT `1` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create index "subscriptions_user_id_feed_id" to table: "subscriptions"
CREATE UNIQUE INDEX `subscriptions_user_id_feed_id` ON `subscriptions` (`user_id`, `feed_id`);

CREATE TRIGGER set_updated_at_subscriptions AFTER
UPDATE ON subscriptions FOR EACH ROW BEGIN
UPDATE subscriptions
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

-- Create "subscription_entries" table
CREATE TABLE `subscription_entries` (
  `id` TEXT NOT NULL,
  `has_read` INTEGER NOT NULL DEFAULT 0,
  `subscription_id` TEXT NOT NULL,
  `feed_entry_id` INTEGER NOT NULL,
  `user_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `1` FOREIGN KEY (`feed_entry_id`) REFERENCES `feed_entries` (`id`) ON UPDATE NO ACTION ON DELETE RESTRICT,
  CONSTRAINT `2` FOREIGN KEY (`subscription_id`) REFERENCES `subscriptions` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create index "subscription_entries_subscription_id_feed_entry_id" to table: "subscription_entries"
CREATE UNIQUE INDEX `subscription_entries_subscription_id_feed_entry_id` ON `subscription_entries` (`subscription_id`, `feed_entry_id`);

CREATE TRIGGER set_updated_at_subscription_entries AFTER
UPDATE ON subscription_entries FOR EACH ROW BEGIN
UPDATE subscription_entries
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

-- Create "subscription_tags" table
CREATE TABLE `subscription_tags` (
  `subscription_id` TEXT NOT NULL,
  `tag_id` TEXT NOT NULL,
  `user_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (`subscription_id`, `tag_id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `1` FOREIGN KEY (`tag_id`) REFERENCES `tags` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `2` FOREIGN KEY (`subscription_id`) REFERENCES `subscriptions` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

CREATE TRIGGER set_updated_at_subscription_tags AFTER
UPDATE ON subscription_tags FOR EACH ROW BEGIN
UPDATE subscription_tags
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

-- Enable back the enforcement of foreign-keys constraints
PRAGMA foreign_keys = ON;
