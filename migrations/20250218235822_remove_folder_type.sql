-- Modify "folders" table
ALTER TABLE "public"."folders"
DROP COLUMN "folder_type";

-- Drop enum type "folder_type"
DROP TYPE "public"."folder_type";
