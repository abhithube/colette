-- Rename a column from "archived_url" to "archived_path"
ALTER TABLE "public"."bookmarks"
RENAME COLUMN "archived_url" TO "archived_path";
