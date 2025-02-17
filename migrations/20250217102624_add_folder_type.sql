-- Create enum type "folder_type"
CREATE TYPE "public"."folder_type" AS ENUM('feeds', 'collections');

-- Modify "folders" table
ALTER TABLE "public"."folders"
DROP CONSTRAINT "folders_user_id_parent_id_title_key",
ADD COLUMN "folder_type" "public"."folder_type" NOT NULL,
ADD CONSTRAINT "folders_user_id_parent_id_title_key" UNIQUE NULLS NOT DISTINCT ("user_id", "parent_id", "title");

-- Modify "bookmarks" table
ALTER TABLE "public"."bookmarks"
DROP COLUMN "folder_id",
ADD COLUMN "collection_id" uuid NULL,
ADD CONSTRAINT "bookmarks_collection_id_fkey" FOREIGN KEY ("collection_id") REFERENCES "public"."collections" ("id") ON UPDATE NO ACTION ON DELETE CASCADE;
