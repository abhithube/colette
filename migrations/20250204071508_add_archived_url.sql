-- Modify "bookmarks" table
ALTER TABLE "public"."bookmarks"
ADD COLUMN "archived_url" TEXT NULL;
