-- Create "users" table
CREATE TABLE `users` (
  `id` TEXT NOT NULL,
  `email` TEXT NOT NULL,
  `display_name` TEXT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (`id`)
);

-- Create index "users_email" to table: "users"
CREATE UNIQUE INDEX `users_email` ON `users` (`email`);

-- Create "accounts" table
CREATE TABLE `accounts` (
  `provider_id` TEXT NOT NULL,
  `account_id` TEXT NOT NULL,
  `password_hash` TEXT NULL,
  `user_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (`provider_id`, `account_id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create "api_keys" table
CREATE TABLE `api_keys` (
  `id` TEXT NOT NULL,
  `lookup_hash` TEXT NOT NULL,
  `verification_hash` TEXT NOT NULL,
  `title` TEXT NOT NULL,
  `preview` TEXT NOT NULL,
  `user_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create index "api_keys_lookup_hash" to table: "api_keys"
CREATE UNIQUE INDEX `api_keys_lookup_hash` ON `api_keys` (`lookup_hash`);

-- Create "feeds" table
CREATE TABLE `feeds` (
  `id` INTEGER NOT NULL,
  `link` TEXT NOT NULL,
  `xml_url` TEXT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (`id`)
);

-- Create index "feeds_link" to table: "feeds"
CREATE UNIQUE INDEX `feeds_link` ON `feeds` (`link`);

-- Create "feed_entries" table
CREATE TABLE `feed_entries` (
  `id` INTEGER NOT NULL,
  `link` TEXT NOT NULL,
  `title` TEXT NOT NULL,
  `published_at` INTEGER NOT NULL,
  `description` TEXT NULL,
  `author` TEXT NULL,
  `thumbnail_url` TEXT NULL,
  `feed_id` INTEGER NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`feed_id`) REFERENCES `feeds` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create index "feed_entries_link" to table: "feed_entries"
CREATE UNIQUE INDEX `feed_entries_link` ON `feed_entries` (`link`);

-- Create index "feed_entries_feed_id_link" to table: "feed_entries"
CREATE UNIQUE INDEX `feed_entries_feed_id_link` ON `feed_entries` (`feed_id`, `link`);

-- Create "user_feeds" table
CREATE TABLE `user_feeds` (
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

-- Create index "user_feeds_user_id_feed_id" to table: "user_feeds"
CREATE UNIQUE INDEX `user_feeds_user_id_feed_id` ON `user_feeds` (`user_id`, `feed_id`);

-- Create "user_feed_entries" table
CREATE TABLE `user_feed_entries` (
  `id` TEXT NOT NULL,
  `has_read` INTEGER NOT NULL DEFAULT 0,
  `user_feed_id` TEXT NOT NULL,
  `feed_entry_id` INTEGER NOT NULL,
  `user_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `1` FOREIGN KEY (`feed_entry_id`) REFERENCES `feed_entries` (`id`) ON UPDATE NO ACTION ON DELETE RESTRICT,
  CONSTRAINT `2` FOREIGN KEY (`user_feed_id`) REFERENCES `user_feeds` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create index "user_feed_entries_user_feed_id_feed_entry_id" to table: "user_feed_entries"
CREATE UNIQUE INDEX `user_feed_entries_user_feed_id_feed_entry_id` ON `user_feed_entries` (`user_feed_id`, `feed_entry_id`);

-- Create "bookmarks" table
CREATE TABLE `bookmarks` (
  `id` TEXT NOT NULL,
  `link` TEXT NOT NULL,
  `title` TEXT NOT NULL,
  `thumbnail_url` TEXT NULL,
  `published_at` INTEGER NULL,
  `author` TEXT NULL,
  `archived_path` TEXT NULL,
  `user_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create index "bookmarks_user_id_link" to table: "bookmarks"
CREATE UNIQUE INDEX `bookmarks_user_id_link` ON `bookmarks` (`user_id`, `link`);

-- Create "tags" table
CREATE TABLE `tags` (
  `id` TEXT NOT NULL,
  `title` TEXT NOT NULL,
  `user_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create index "tags_user_id_title" to table: "tags"
CREATE UNIQUE INDEX `tags_user_id_title` ON `tags` (`user_id`, `title`);

-- Create "user_feed_tags" table
CREATE TABLE `user_feed_tags` (
  `user_feed_id` TEXT NOT NULL,
  `tag_id` TEXT NOT NULL,
  `user_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (`user_feed_id`, `tag_id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `1` FOREIGN KEY (`tag_id`) REFERENCES `tags` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `2` FOREIGN KEY (`user_feed_id`) REFERENCES `user_feeds` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create "bookmark_tags" table
CREATE TABLE `bookmark_tags` (
  `bookmark_id` TEXT NOT NULL,
  `tag_id` TEXT NOT NULL,
  `user_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (`bookmark_id`, `tag_id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `1` FOREIGN KEY (`tag_id`) REFERENCES `tags` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `2` FOREIGN KEY (`bookmark_id`) REFERENCES `bookmarks` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create "streams" table
CREATE TABLE `streams` (
  `id` TEXT NOT NULL,
  `title` TEXT NOT NULL,
  `description` TEXT NULL,
  `filter_raw` TEXT NOT NULL,
  `user_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create index "streams_user_id_title" to table: "streams"
CREATE UNIQUE INDEX `streams_user_id_title` ON `streams` (`user_id`, `title`);

-- Create "collections" table
CREATE TABLE `collections` (
  `id` TEXT NOT NULL,
  `title` TEXT NOT NULL,
  `description` TEXT NULL,
  `filter_raw` TEXT NOT NULL,
  `user_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create index "collections_user_id_title" to table: "collections"
CREATE UNIQUE INDEX `collections_user_id_title` ON `collections` (`user_id`, `title`);
