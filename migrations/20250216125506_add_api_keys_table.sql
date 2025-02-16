-- Create "api_keys" table
CREATE TABLE "public"."api_keys" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid (),
  "value_hash" TEXT NOT NULL,
  "value_preview" TEXT NOT NULL,
  "title" TEXT NOT NULL,
  "user_id" uuid NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "api_keys_value_hash_key" UNIQUE ("value_hash"),
  CONSTRAINT "api_keys_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
