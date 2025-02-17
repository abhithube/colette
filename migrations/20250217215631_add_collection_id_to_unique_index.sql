-- Modify "bookmarks" table
ALTER TABLE "public"."bookmarks"
DROP CONSTRAINT "bookmarks_user_id_link_key",
ADD CONSTRAINT "bookmarks_user_id_collection_id_link_key" UNIQUE NULLS NOT DISTINCT ("user_id", "collection_id", "link");
