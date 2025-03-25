-- Disable the enforcement of foreign-keys constraints
PRAGMA foreign_keys = off;

-- Create "new_users" table
CREATE TABLE `new_users` (
  `id` TEXT NOT NULL,
  `name` TEXT NULL,
  `email` TEXT NOT NULL,
  `verified_at` INTEGER NULL,
  `password_hash` TEXT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (unixepoch()),
  `updated_at` INTEGER NOT NULL DEFAULT (unixepoch()),
  PRIMARY KEY (`id`)
);

-- Copy rows from old table "users" to new temporary table "new_users"
INSERT INTO
  `new_users` (
    `id`,
    `name`,
    `email`,
    `verified_at`,
    `password_hash`,
    `created_at`,
    `updated_at`
  )
SELECT
  `id`,
  `name`,
  `email`,
  `verified_at`,
  `password_hash`,
  ifnull(`created_at`, (unixepoch())) AS `created_at`,
  ifnull(`updated_at`, (unixepoch())) AS `updated_at`
FROM
  `users`;

-- Drop "users" table after copying rows
DROP TABLE `users`;

-- Rename temporary table "new_users" to "users"
ALTER TABLE `new_users`
RENAME TO `users`;

-- Create index "users_email" to table: "users"
CREATE UNIQUE INDEX `users_email` ON `users` (`email`);

-- Create "new_api_keys" table
CREATE TABLE `new_api_keys` (
  `id` TEXT NOT NULL,
  `lookup_hash` TEXT NOT NULL,
  `verification_hash` TEXT NOT NULL,
  `title` TEXT NOT NULL,
  `preview` TEXT NOT NULL,
  `user_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (unixepoch()),
  `updated_at` INTEGER NOT NULL DEFAULT (unixepoch()),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Copy rows from old table "api_keys" to new temporary table "new_api_keys"
INSERT INTO
  `new_api_keys` (
    `id`,
    `lookup_hash`,
    `verification_hash`,
    `title`,
    `preview`,
    `user_id`,
    `created_at`,
    `updated_at`
  )
SELECT
  `id`,
  `lookup_hash`,
  `verification_hash`,
  `title`,
  `preview`,
  `user_id`,
  ifnull(`created_at`, (unixepoch())) AS `created_at`,
  ifnull(`updated_at`, (unixepoch())) AS `updated_at`
FROM
  `api_keys`;

-- Drop "api_keys" table after copying rows
DROP TABLE `api_keys`;

-- Rename temporary table "new_api_keys" to "api_keys"
ALTER TABLE `new_api_keys`
RENAME TO `api_keys`;

-- Create index "api_keys_lookup_hash" to table: "api_keys"
CREATE UNIQUE INDEX `api_keys_lookup_hash` ON `api_keys` (`lookup_hash`);

-- Create "new_feeds" table
CREATE TABLE `new_feeds` (
  `id` TEXT NOT NULL,
  `link` TEXT NOT NULL,
  `xml_url` TEXT NULL,
  `title` TEXT NOT NULL,
  `description` TEXT NULL,
  `refreshed_at` INTEGER NULL,
  PRIMARY KEY (`id`)
);

-- Copy rows from old table "feeds" to new temporary table "new_feeds"
INSERT INTO
  `new_feeds` (
    `id`,
    `link`,
    `xml_url`,
    `title`,
    `description`,
    `refreshed_at`
  )
SELECT
  `id`,
  `link`,
  `xml_url`,
  `title`,
  `description`,
  `refreshed_at`
FROM
  `feeds`;

-- Drop "feeds" table after copying rows
DROP TABLE `feeds`;

-- Rename temporary table "new_feeds" to "feeds"
ALTER TABLE `new_feeds`
RENAME TO `feeds`;

-- Create index "feeds_link" to table: "feeds"
CREATE UNIQUE INDEX `feeds_link` ON `feeds` (`link`);

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

-- Create "new_subscriptions" table
CREATE TABLE `new_subscriptions` (
  `id` TEXT NOT NULL,
  `title` TEXT NOT NULL,
  `user_id` TEXT NOT NULL,
  `feed_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (unixepoch()),
  `updated_at` INTEGER NOT NULL DEFAULT (unixepoch()),
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
  ifnull(`created_at`, (unixepoch())) AS `created_at`,
  ifnull(`updated_at`, (unixepoch())) AS `updated_at`
FROM
  `subscriptions`;

-- Drop "subscriptions" table after copying rows
DROP TABLE `subscriptions`;

-- Rename temporary table "new_subscriptions" to "subscriptions"
ALTER TABLE `new_subscriptions`
RENAME TO `subscriptions`;

-- Create index "subscriptions_user_id_feed_id" to table: "subscriptions"
CREATE UNIQUE INDEX `subscriptions_user_id_feed_id` ON `subscriptions` (`user_id`, `feed_id`);

-- Create "new_read_entries" table
CREATE TABLE `new_read_entries` (
  `subscription_id` TEXT NOT NULL,
  `feed_entry_id` TEXT NOT NULL,
  `user_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (unixepoch()),
  PRIMARY KEY (`subscription_id`, `feed_entry_id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `1` FOREIGN KEY (`feed_entry_id`) REFERENCES `feed_entries` (`id`) ON UPDATE NO ACTION ON DELETE RESTRICT,
  CONSTRAINT `2` FOREIGN KEY (`subscription_id`) REFERENCES `subscriptions` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Copy rows from old table "read_entries" to new temporary table "new_read_entries"
INSERT INTO
  `new_read_entries` (
    `subscription_id`,
    `feed_entry_id`,
    `user_id`,
    `created_at`
  )
SELECT
  `subscription_id`,
  `feed_entry_id`,
  `user_id`,
  ifnull(`created_at`, (unixepoch())) AS `created_at`
FROM
  `read_entries`;

-- Drop "read_entries" table after copying rows
DROP TABLE `read_entries`;

-- Rename temporary table "new_read_entries" to "read_entries"
ALTER TABLE `new_read_entries`
RENAME TO `read_entries`;

-- Create "new_bookmarks" table
CREATE TABLE `new_bookmarks` (
  `id` TEXT NOT NULL,
  `link` TEXT NOT NULL,
  `title` TEXT NOT NULL,
  `thumbnail_url` TEXT NULL,
  `published_at` INTEGER NULL,
  `author` TEXT NULL,
  `archived_path` TEXT NULL,
  `user_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (unixepoch()),
  `updated_at` INTEGER NOT NULL DEFAULT (unixepoch()),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Copy rows from old table "bookmarks" to new temporary table "new_bookmarks"
INSERT INTO
  `new_bookmarks` (
    `id`,
    `link`,
    `title`,
    `thumbnail_url`,
    `published_at`,
    `author`,
    `archived_path`,
    `user_id`,
    `created_at`,
    `updated_at`
  )
SELECT
  `id`,
  `link`,
  `title`,
  `thumbnail_url`,
  `published_at`,
  `author`,
  `archived_path`,
  `user_id`,
  ifnull(`created_at`, (unixepoch())) AS `created_at`,
  ifnull(`updated_at`, (unixepoch())) AS `updated_at`
FROM
  `bookmarks`;

-- Drop "bookmarks" table after copying rows
DROP TABLE `bookmarks`;

-- Rename temporary table "new_bookmarks" to "bookmarks"
ALTER TABLE `new_bookmarks`
RENAME TO `bookmarks`;

-- Create index "bookmarks_user_id_link" to table: "bookmarks"
CREATE UNIQUE INDEX `bookmarks_user_id_link` ON `bookmarks` (`user_id`, `link`);

-- Create "new_tags" table
CREATE TABLE `new_tags` (
  `id` TEXT NOT NULL,
  `title` TEXT NOT NULL,
  `user_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (unixepoch()),
  `updated_at` INTEGER NOT NULL DEFAULT (unixepoch()),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Copy rows from old table "tags" to new temporary table "new_tags"
INSERT INTO
  `new_tags` (
    `id`,
    `title`,
    `user_id`,
    `created_at`,
    `updated_at`
  )
SELECT
  `id`,
  `title`,
  `user_id`,
  ifnull(`created_at`, (unixepoch())) AS `created_at`,
  ifnull(`updated_at`, (unixepoch())) AS `updated_at`
FROM
  `tags`;

-- Drop "tags" table after copying rows
DROP TABLE `tags`;

-- Rename temporary table "new_tags" to "tags"
ALTER TABLE `new_tags`
RENAME TO `tags`;

-- Create index "tags_user_id_title" to table: "tags"
CREATE UNIQUE INDEX `tags_user_id_title` ON `tags` (`user_id`, `title`);

-- Create "new_subscription_tags" table
CREATE TABLE `new_subscription_tags` (
  `subscription_id` TEXT NOT NULL,
  `tag_id` TEXT NOT NULL,
  `user_id` TEXT NOT NULL,
  PRIMARY KEY (`subscription_id`, `tag_id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `1` FOREIGN KEY (`tag_id`) REFERENCES `tags` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `2` FOREIGN KEY (`subscription_id`) REFERENCES `subscriptions` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Copy rows from old table "subscription_tags" to new temporary table "new_subscription_tags"
INSERT INTO
  `new_subscription_tags` (`subscription_id`, `tag_id`, `user_id`)
SELECT
  `subscription_id`,
  `tag_id`,
  `user_id`
FROM
  `subscription_tags`;

-- Drop "subscription_tags" table after copying rows
DROP TABLE `subscription_tags`;

-- Rename temporary table "new_subscription_tags" to "subscription_tags"
ALTER TABLE `new_subscription_tags`
RENAME TO `subscription_tags`;

-- Create "new_bookmark_tags" table
CREATE TABLE `new_bookmark_tags` (
  `bookmark_id` TEXT NOT NULL,
  `tag_id` TEXT NOT NULL,
  `user_id` TEXT NOT NULL,
  PRIMARY KEY (`bookmark_id`, `tag_id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `1` FOREIGN KEY (`tag_id`) REFERENCES `tags` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `2` FOREIGN KEY (`bookmark_id`) REFERENCES `bookmarks` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Copy rows from old table "bookmark_tags" to new temporary table "new_bookmark_tags"
INSERT INTO
  `new_bookmark_tags` (`bookmark_id`, `tag_id`, `user_id`)
SELECT
  `bookmark_id`,
  `tag_id`,
  `user_id`
FROM
  `bookmark_tags`;

-- Drop "bookmark_tags" table after copying rows
DROP TABLE `bookmark_tags`;

-- Rename temporary table "new_bookmark_tags" to "bookmark_tags"
ALTER TABLE `new_bookmark_tags`
RENAME TO `bookmark_tags`;

-- Create "new_streams" table
CREATE TABLE `new_streams` (
  `id` TEXT NOT NULL,
  `title` TEXT NOT NULL,
  `description` TEXT NULL,
  `filter_raw` TEXT NOT NULL,
  `user_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (unixepoch()),
  `updated_at` INTEGER NOT NULL DEFAULT (unixepoch()),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Copy rows from old table "streams" to new temporary table "new_streams"
INSERT INTO
  `new_streams` (
    `id`,
    `title`,
    `description`,
    `filter_raw`,
    `user_id`,
    `created_at`,
    `updated_at`
  )
SELECT
  `id`,
  `title`,
  `description`,
  `filter_raw`,
  `user_id`,
  ifnull(`created_at`, (unixepoch())) AS `created_at`,
  ifnull(`updated_at`, (unixepoch())) AS `updated_at`
FROM
  `streams`;

-- Drop "streams" table after copying rows
DROP TABLE `streams`;

-- Rename temporary table "new_streams" to "streams"
ALTER TABLE `new_streams`
RENAME TO `streams`;

-- Create index "streams_user_id_title" to table: "streams"
CREATE UNIQUE INDEX `streams_user_id_title` ON `streams` (`user_id`, `title`);

-- Create "new_collections" table
CREATE TABLE `new_collections` (
  `id` TEXT NOT NULL,
  `title` TEXT NOT NULL,
  `description` TEXT NULL,
  `filter_raw` TEXT NOT NULL,
  `user_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (unixepoch()),
  `updated_at` INTEGER NOT NULL DEFAULT (unixepoch()),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Copy rows from old table "collections" to new temporary table "new_collections"
INSERT INTO
  `new_collections` (
    `id`,
    `title`,
    `description`,
    `filter_raw`,
    `user_id`,
    `created_at`,
    `updated_at`
  )
SELECT
  `id`,
  `title`,
  `description`,
  `filter_raw`,
  `user_id`,
  ifnull(`created_at`, (unixepoch())) AS `created_at`,
  ifnull(`updated_at`, (unixepoch())) AS `updated_at`
FROM
  `collections`;

-- Drop "collections" table after copying rows
DROP TABLE `collections`;

-- Rename temporary table "new_collections" to "collections"
ALTER TABLE `new_collections`
RENAME TO `collections`;

-- Create index "collections_user_id_title" to table: "collections"
CREATE UNIQUE INDEX `collections_user_id_title` ON `collections` (`user_id`, `title`);

-- Create "new_sessions" table
CREATE TABLE `new_sessions` (
  `id` INTEGER NOT NULL,
  `token` TEXT NOT NULL,
  `user_agent` TEXT NULL,
  `ip_address` TEXT NULL,
  `expires_at` INTEGER NOT NULL,
  `user_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (unixepoch()),
  `updated_at` INTEGER NOT NULL DEFAULT (unixepoch()),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Copy rows from old table "sessions" to new temporary table "new_sessions"
INSERT INTO
  `new_sessions` (
    `id`,
    `token`,
    `user_agent`,
    `ip_address`,
    `expires_at`,
    `user_id`,
    `created_at`,
    `updated_at`
  )
SELECT
  `id`,
  `token`,
  `user_agent`,
  `ip_address`,
  `expires_at`,
  `user_id`,
  ifnull(`created_at`, (unixepoch())) AS `created_at`,
  ifnull(`updated_at`, (unixepoch())) AS `updated_at`
FROM
  `sessions`;

-- Drop "sessions" table after copying rows
DROP TABLE `sessions`;

-- Rename temporary table "new_sessions" to "sessions"
ALTER TABLE `new_sessions`
RENAME TO `sessions`;

-- Create "new_jobs" table
CREATE TABLE `new_jobs` (
  `id` TEXT NOT NULL,
  `job_type` TEXT NOT NULL,
  `data` BLOB NOT NULL,
  `status` TEXT NOT NULL DEFAULT 'pending',
  `group_id` TEXT NULL,
  `message` TEXT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (unixepoch()),
  `completed_at` INTEGER NULL,
  PRIMARY KEY (`id`)
);

-- Copy rows from old table "jobs" to new temporary table "new_jobs"
INSERT INTO
  `new_jobs` (
    `id`,
    `job_type`,
    `data`,
    `status`,
    `group_id`,
    `message`,
    `created_at`,
    `completed_at`
  )
SELECT
  `id`,
  `job_type`,
  `data`,
  `status`,
  `group_id`,
  `message`,
  ifnull(`created_at`, (unixepoch())) AS `created_at`,
  `completed_at`
FROM
  `jobs`;

-- Drop "jobs" table after copying rows
DROP TABLE `jobs`;

-- Rename temporary table "new_jobs" to "jobs"
ALTER TABLE `new_jobs`
RENAME TO `jobs`;

-- Enable back the enforcement of foreign-keys constraints
PRAGMA foreign_keys = ON;
