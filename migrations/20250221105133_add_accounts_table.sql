-- Modify "users" table
ALTER TABLE "public"."users"
DROP COLUMN "password",
ADD COLUMN "display_name" TEXT NULL;

-- Create "accounts" table
CREATE TABLE "public"."accounts" (
  "provider_id" TEXT NOT NULL,
  "account_id" TEXT NOT NULL,
  "password_hash" TEXT NULL,
  "user_id" uuid NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT "accounts_provider_id_account_id_key" UNIQUE ("provider_id", "account_id"),
  CONSTRAINT "accounts_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
