-- Rename a column from "value_hash" to "verification_hash"
ALTER TABLE "public"."api_keys"
RENAME COLUMN "value_hash" TO "verification_hash";

-- Rename a column from "value_preview" to "preview"
ALTER TABLE "public"."api_keys"
RENAME COLUMN "value_preview" TO "preview";

-- Modify "api_keys" table
ALTER TABLE "public"."api_keys"
DROP CONSTRAINT "api_keys_value_hash_key",
ADD COLUMN "lookup_hash" TEXT NOT NULL,
ADD CONSTRAINT "api_keys_lookup_hash_key" UNIQUE ("lookup_hash");
