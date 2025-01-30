-- Modify "feeds" table
ALTER TABLE "public"."feeds"
DROP COLUMN "title";

-- Modify "user_feeds" table
ALTER TABLE "public"."user_feeds"
ALTER COLUMN "title"
SET NOT NULL;
