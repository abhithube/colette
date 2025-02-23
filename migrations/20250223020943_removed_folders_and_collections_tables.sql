-- Modify "bookmarks" table
ALTER TABLE "public"."bookmarks"
DROP COLUMN "collection_id",
ADD CONSTRAINT "bookmarks_user_id_link_key" UNIQUE ("user_id", "link");

-- Modify "collections" table
ALTER TABLE "public"."collections"
DROP CONSTRAINT "collections_folder_id_fkey",
DROP CONSTRAINT "collections_user_id_fkey";

-- Modify "folders" table
ALTER TABLE "public"."folders"
DROP CONSTRAINT "folders_user_id_fkey";

-- Modify "user_feeds" table
ALTER TABLE "public"."user_feeds"
DROP COLUMN "folder_id";

-- Drop "collections" table
DROP TABLE "public"."collections";

-- Drop "folders" table
DROP TABLE "public"."folders";
