-- Disable the enforcement of foreign-keys constraints
PRAGMA foreign_keys = off;

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
  `new_feeds` (`id`, `link`, `xml_url`)
SELECT
  `id`,
  `link`,
  `xml_url`
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
  `id` INTEGER NOT NULL,
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

-- Create index "feed_entries_link" to table: "feed_entries"
CREATE UNIQUE INDEX `feed_entries_link` ON `feed_entries` (`link`);

-- Create index "feed_entries_feed_id_link" to table: "feed_entries"
CREATE UNIQUE INDEX `feed_entries_feed_id_link` ON `feed_entries` (`feed_id`, `link`);

-- Enable back the enforcement of foreign-keys constraints
PRAGMA foreign_keys = ON;
