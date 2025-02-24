-- Create "collections" table
CREATE TABLE "public"."collections" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid (),
  "title" TEXT NOT NULL,
  "description" TEXT NULL,
  "filter" jsonb NOT NULL,
  "user_id" uuid NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "collections_user_id_title_key" UNIQUE ("user_id", "title"),
  CONSTRAINT "collections_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create "streams" table
CREATE TABLE "public"."streams" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid (),
  "title" TEXT NOT NULL,
  "description" TEXT NULL,
  "filter" jsonb NOT NULL,
  "user_id" uuid NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "streams_user_id_title_key" UNIQUE ("user_id", "title"),
  CONSTRAINT "streams_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
