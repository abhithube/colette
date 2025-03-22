-- Create "jobs" table
CREATE TABLE `jobs` (
  `id` TEXT NOT NULL,
  `job_type` TEXT NOT NULL,
  `data` BLOB NOT NULL,
  `status` TEXT NOT NULL DEFAULT 'pending',
  `group_id` TEXT NULL,
  `message` TEXT NULL,
  `created_at` TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `completed_at` TEXT NULL,
  PRIMARY KEY (`id`)
);
