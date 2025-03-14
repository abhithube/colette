-- Disable the enforcement of foreign-keys constraints
PRAGMA foreign_keys = off;

-- Rename a column from "display_name" to "name"
ALTER TABLE `users`
RENAME COLUMN `display_name` TO `name`;

-- Add column "verified_at" to table: "users"
ALTER TABLE `users`
ADD COLUMN `verified_at` TEXT NULL;

-- Add column "password_hash" to table: "users"
ALTER TABLE `users`
ADD COLUMN `password_hash` TEXT NULL;

-- Drop "accounts" table
DROP TABLE `accounts`;

-- Create "sessions" table
CREATE TABLE `sessions` (
  `id` INTEGER NOT NULL,
  `token` TEXT NOT NULL,
  `user_agent` TEXT NULL,
  `ip_address` TEXT NULL,
  `expires_at` TEXT NOT NULL,
  `user_id` TEXT NOT NULL,
  `created_at` TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Enable back the enforcement of foreign-keys constraints
PRAGMA foreign_keys = ON;
