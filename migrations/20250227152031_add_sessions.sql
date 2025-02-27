-- Create "sessions" table
CREATE TABLE `sessions` (
  `id` TEXT NOT NULL,
  `data` BLOB NOT NULL,
  `expires_at` TEXT NOT NULL,
  PRIMARY KEY (`id`)
);
