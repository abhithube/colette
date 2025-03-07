-- Disable the enforcement of foreign-keys constraints
PRAGMA foreign_keys = off;

-- Drop "subscription_entries" table
DROP TABLE `subscription_entries`;

-- Create "new_feed_entries" table
CREATE TABLE `new_feed_entries` (
  `id` TEXT NOT NULL,
  `link` TEXT NOT NULL,
  `title` TEXT NOT NULL,
  `published_at` INTEGER NOT NULL,
  `description` TEXT NULL,
  `author` TEXT NULL,
  `thumbnail_url` TEXT NULL,
  `feed_id` TEXT NOT NULL,
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`feed_id`) REFERENCES `feeds` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Copy rows from old table "feed_entries" to new temporary table "new_feed_entries"
INSERT INTO
  `new_feed_entries` (
    `id`,
    `link`,
    `title`,
    `published_at`,
    `description`,
    `author`,
    `thumbnail_url`,
    `feed_id`
  )
SELECT
  `id`,
  `link`,
  `title`,
  `published_at`,
  `description`,
  `author`,
  `thumbnail_url`,
  `feed_id`
FROM
  `feed_entries`;

-- Drop "feed_entries" table after copying rows
DROP TABLE `feed_entries`;

-- Rename temporary table "new_feed_entries" to "feed_entries"
ALTER TABLE `new_feed_entries`
RENAME TO `feed_entries`;

-- Create index "feed_entries_feed_id_link" to table: "feed_entries"
CREATE UNIQUE INDEX `feed_entries_feed_id_link` ON `feed_entries` (`feed_id`, `link`);

-- Create "read_entries" table
CREATE TABLE `read_entries` (
  `subscription_id` TEXT NOT NULL,
  `feed_entry_id` TEXT NOT NULL,
  `user_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (`subscription_id`, `feed_entry_id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `1` FOREIGN KEY (`feed_entry_id`) REFERENCES `feed_entries` (`id`) ON UPDATE NO ACTION ON DELETE RESTRICT,
  CONSTRAINT `2` FOREIGN KEY (`subscription_id`) REFERENCES `subscriptions` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Enable back the enforcement of foreign-keys constraints
PRAGMA foreign_keys = ON;
