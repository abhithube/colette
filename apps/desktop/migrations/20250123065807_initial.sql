-- Create "users" table
CREATE TABLE `users` (`id` text NOT NULL, `email` text NOT NULL, `password` text NOT NULL, `created_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), `updated_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), PRIMARY KEY (`id`));
-- Create index "users_email" to table: "users"
CREATE UNIQUE INDEX `users_email` ON `users` (`email`);
-- Create "feeds" table
CREATE TABLE `feeds` (`id` text NOT NULL, `link` text NOT NULL, `title` text NOT NULL, `xml_url` text NULL, `created_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), `updated_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), PRIMARY KEY (`id`));
-- Create index "feeds_link" to table: "feeds"
CREATE UNIQUE INDEX `feeds_link` ON `feeds` (`link`);
-- Create "feed_entries" table
CREATE TABLE `feed_entries` (`id` text NOT NULL, `link` text NOT NULL, `title` text NOT NULL, `published_at` text NOT NULL, `description` text NULL, `author` text NULL, `thumbnail_url` text NULL, `feed_id` text NOT NULL, `created_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), `updated_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), PRIMARY KEY (`id`), CONSTRAINT `0` FOREIGN KEY (`feed_id`) REFERENCES `feeds` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE);
-- Create index "feed_entries_link" to table: "feed_entries"
CREATE UNIQUE INDEX `feed_entries_link` ON `feed_entries` (`link`);
-- Create index "feed_entries_feed_id_link" to table: "feed_entries"
CREATE UNIQUE INDEX `feed_entries_feed_id_link` ON `feed_entries` (`feed_id`, `link`);
-- Create "bookmarks" table
CREATE TABLE `bookmarks` (`id` text NOT NULL, `link` text NOT NULL, `title` text NOT NULL, `thumbnail_url` text NULL, `published_at` text NULL, `author` text NULL, `created_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), `updated_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), PRIMARY KEY (`id`));
-- Create index "bookmarks_link" to table: "bookmarks"
CREATE UNIQUE INDEX `bookmarks_link` ON `bookmarks` (`link`);
-- Create "folders" table
CREATE TABLE `folders` (`id` text NOT NULL, `title` text NOT NULL, `parent_id` text NULL, `user_id` text NOT NULL, `created_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), `updated_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), PRIMARY KEY (`id`), CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE, CONSTRAINT `1` FOREIGN KEY (`parent_id`) REFERENCES `folders` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE);
-- Create index "folders_user_id_parent_id_title" to table: "folders"
CREATE UNIQUE INDEX `folders_user_id_parent_id_title` ON `folders` (`user_id`, `parent_id`, `title`);
-- Create "user_feeds" table
CREATE TABLE `user_feeds` (`id` text NOT NULL, `title` text NULL, `folder_id` text NULL, `user_id` text NOT NULL, `feed_id` text NOT NULL, `created_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), `updated_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), PRIMARY KEY (`id`), CONSTRAINT `0` FOREIGN KEY (`feed_id`) REFERENCES `feeds` (`id`) ON UPDATE NO ACTION ON DELETE RESTRICT, CONSTRAINT `1` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE, CONSTRAINT `2` FOREIGN KEY (`folder_id`) REFERENCES `folders` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE);
-- Create index "user_feeds_user_id_feed_id" to table: "user_feeds"
CREATE UNIQUE INDEX `user_feeds_user_id_feed_id` ON `user_feeds` (`user_id`, `feed_id`);
-- Create "user_feed_entries" table
CREATE TABLE `user_feed_entries` (`id` text NOT NULL, `has_read` integer NOT NULL DEFAULT 0, `user_feed_id` text NOT NULL, `feed_entry_id` text NOT NULL, `user_id` text NOT NULL, `created_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), `updated_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), PRIMARY KEY (`id`), CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE, CONSTRAINT `1` FOREIGN KEY (`feed_entry_id`) REFERENCES `feed_entries` (`id`) ON UPDATE NO ACTION ON DELETE RESTRICT, CONSTRAINT `2` FOREIGN KEY (`user_feed_id`) REFERENCES `user_feeds` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE);
-- Create index "user_feed_entries_user_feed_id_feed_entry_id" to table: "user_feed_entries"
CREATE UNIQUE INDEX `user_feed_entries_user_feed_id_feed_entry_id` ON `user_feed_entries` (`user_feed_id`, `feed_entry_id`);
-- Create "user_bookmarks" table
CREATE TABLE `user_bookmarks` (`id` text NOT NULL, `title` text NULL, `thumbnail_url` text NULL, `published_at` TIMESTAMPTZ NULL, `author` text NULL, `folder_id` text NULL, `user_id` text NOT NULL, `bookmark_id` text NOT NULL, `created_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), `updated_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), PRIMARY KEY (`id`), CONSTRAINT `0` FOREIGN KEY (`bookmark_id`) REFERENCES `bookmarks` (`id`) ON UPDATE NO ACTION ON DELETE RESTRICT, CONSTRAINT `1` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE, CONSTRAINT `2` FOREIGN KEY (`folder_id`) REFERENCES `folders` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE);
-- Create index "user_bookmarks_user_id_bookmark_id" to table: "user_bookmarks"
CREATE UNIQUE INDEX `user_bookmarks_user_id_bookmark_id` ON `user_bookmarks` (`user_id`, `bookmark_id`);
-- Create "tags" table
CREATE TABLE `tags` (`id` text NOT NULL, `title` text NOT NULL, `user_id` text NOT NULL, `created_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), `updated_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), PRIMARY KEY (`id`), CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE);
-- Create index "tags_user_id_title" to table: "tags"
CREATE UNIQUE INDEX `tags_user_id_title` ON `tags` (`user_id`, `title`);
-- Create "user_feed_tags" table
CREATE TABLE `user_feed_tags` (`user_feed_id` text NOT NULL, `tag_id` text NOT NULL, `user_id` text NOT NULL, `created_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), `updated_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), PRIMARY KEY (`user_feed_id`, `tag_id`), CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE, CONSTRAINT `1` FOREIGN KEY (`tag_id`) REFERENCES `tags` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE, CONSTRAINT `2` FOREIGN KEY (`user_feed_id`) REFERENCES `user_feeds` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE);
-- Create "user_bookmark_tags" table
CREATE TABLE `user_bookmark_tags` (`user_bookmark_id` text NOT NULL, `tag_id` text NOT NULL, `user_id` text NOT NULL, `created_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), `updated_at` text NOT NULL DEFAULT (CURRENT_TIMESTAMP), PRIMARY KEY (`user_bookmark_id`, `tag_id`), CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE, CONSTRAINT `1` FOREIGN KEY (`tag_id`) REFERENCES `tags` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE, CONSTRAINT `2` FOREIGN KEY (`user_bookmark_id`) REFERENCES `user_bookmarks` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE);
