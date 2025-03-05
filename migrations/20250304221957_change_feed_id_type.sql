-- Disable the enforcement of foreign-keys constraints
PRAGMA foreign_keys = off;

-- Create "new_feeds" table
CREATE TABLE `new_feeds` (
  `id` TEXT NOT NULL,
  `link` TEXT NOT NULL,
  `xml_url` TEXT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (`id`)
);

-- Copy rows from old table "feeds" to new temporary table "new_feeds"
INSERT INTO
  `new_feeds` (
    `id`,
    `link`,
    `xml_url`,
    `created_at`,
    `updated_at`
  )
SELECT
  `id`,
  `link`,
  `xml_url`,
  `created_at`,
  `updated_at`
FROM
  `feeds`;

-- Drop "feeds" table after copying rows
DROP TABLE `feeds`;

-- Rename temporary table "new_feeds" to "feeds"
ALTER TABLE `new_feeds`
RENAME TO `feeds`;

-- Create index "feeds_link" to table: "feeds"
CREATE UNIQUE INDEX `feeds_link` ON `feeds` (`link`);

CREATE TRIGGER set_updated_at_feeds AFTER
UPDATE ON feeds FOR EACH ROW BEGIN
UPDATE feeds
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

-- Create "new_feed_entries" table
CREATE TABLE `new_feed_entries` (
  `id` INTEGER NOT NULL,
  `link` TEXT NOT NULL,
  `title` TEXT NOT NULL,
  `published_at` INTEGER NOT NULL,
  `description` TEXT NULL,
  `author` TEXT NULL,
  `thumbnail_url` TEXT NULL,
  `feed_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
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
    `feed_id`,
    `created_at`,
    `updated_at`
  )
SELECT
  `id`,
  `link`,
  `title`,
  `published_at`,
  `description`,
  `author`,
  `thumbnail_url`,
  `feed_id`,
  `created_at`,
  `updated_at`
FROM
  `feed_entries`;

-- Drop "feed_entries" table after copying rows
DROP TABLE `feed_entries`;

-- Rename temporary table "new_feed_entries" to "feed_entries"
ALTER TABLE `new_feed_entries`
RENAME TO `feed_entries`;

-- Create index "feed_entries_link" to table: "feed_entries"
CREATE UNIQUE INDEX `feed_entries_link` ON `feed_entries` (`link`);

-- Create index "feed_entries_feed_id_link" to table: "feed_entries"
CREATE UNIQUE INDEX `feed_entries_feed_id_link` ON `feed_entries` (`feed_id`, `link`);

CREATE TRIGGER set_updated_at_feed_entries AFTER
UPDATE ON feed_entries FOR EACH ROW BEGIN
UPDATE feed_entries
SET
  updated_at = strftime('%s', 'now')
WHERE
  id = OLD.id;

END;

-- Create "new_subscriptions" table
CREATE TABLE `new_subscriptions` (
  `id` TEXT NOT NULL,
  `title` TEXT NOT NULL,
  `user_id` TEXT NOT NULL,
  `feed_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`feed_id`) REFERENCES `feeds` (`id`) ON UPDATE NO ACTION ON DELETE RESTRICT,
  CONSTRAINT `1` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Copy rows from old table "subscriptions" to new temporary table "new_subscriptions"
INSERT INTO
  `new_subscriptions` (
    `id`,
    `title`,
    `user_id`,
    `feed_id`,
    `created_at`,
    `updated_at`
  )
SELECT
  `id`,
  `title`,
  `user_id`,
  `feed_id`,
  `created_at`,
  `updated_at`
FROM
  `subscriptions`;

-- Drop "subscriptions" table after copying rows
DROP TABLE `subscriptions`;

-- Rename temporary table "new_subscriptions" to "subscriptions"
ALTER TABLE `new_subscriptions`
RENAME TO `subscriptions`;

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

-- Enable back the enforcement of foreign-keys constraints
PRAGMA foreign_keys = ON;
