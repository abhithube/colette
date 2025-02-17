-- Create "collections" table
CREATE TABLE "public"."collections" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid (),
  "title" TEXT NOT NULL,
  "folder_id" uuid NULL,
  "user_id" uuid NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "collections_user_id_folder_id_title_key" UNIQUE NULLS NOT DISTINCT ("user_id", "folder_id", "title"),
  CONSTRAINT "collections_folder_id_fkey" FOREIGN KEY ("folder_id") REFERENCES "public"."folders" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "collections_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
