-- Create "users" table
CREATE TABLE "public"."users" (
  "id" uuid NOT NULL,
  "email" text NOT NULL,
  "display_name" text NULL,
  "image_url" text NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "updated_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "users_email_key" UNIQUE ("email")
);
-- Create "jobs" table
CREATE TABLE "public"."jobs" (
  "id" uuid NOT NULL,
  "job_type" text NOT NULL,
  "data_json" jsonb NOT NULL,
  "status" text NOT NULL DEFAULT 'pending',
  "group_identifier" text NULL,
  "message" text NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "completed_at" timestamptz NULL,
  PRIMARY KEY ("id")
);
-- Create "accounts" table
CREATE TABLE "public"."accounts" (
  "id" uuid NOT NULL,
  "sub" text NOT NULL,
  "provider" text NOT NULL,
  "password_hash" text NULL,
  "user_id" uuid NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "updated_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "accounts_provider_sub_key" UNIQUE ("provider", "sub"),
  CONSTRAINT "accounts_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "api_keys" table
CREATE TABLE "public"."api_keys" (
  "id" uuid NOT NULL,
  "lookup_hash" text NOT NULL,
  "verification_hash" text NOT NULL,
  "title" text NOT NULL,
  "preview" text NOT NULL,
  "user_id" uuid NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "updated_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "api_keys_lookup_hash_key" UNIQUE ("lookup_hash"),
  CONSTRAINT "api_keys_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "bookmarks" table
CREATE TABLE "public"."bookmarks" (
  "id" uuid NOT NULL,
  "link" text NOT NULL,
  "title" text NOT NULL,
  "thumbnail_url" text NULL,
  "published_at" timestamptz NULL,
  "author" text NULL,
  "archived_path" text NULL,
  "user_id" uuid NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "updated_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "bookmarks_user_id_link_key" UNIQUE ("user_id", "link"),
  CONSTRAINT "bookmarks_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "tags" table
CREATE TABLE "public"."tags" (
  "id" uuid NOT NULL,
  "title" text NOT NULL,
  "user_id" uuid NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "updated_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "tags_user_id_title_key" UNIQUE ("user_id", "title"),
  CONSTRAINT "tags_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "bookmark_tags" table
CREATE TABLE "public"."bookmark_tags" (
  "bookmark_id" uuid NOT NULL,
  "tag_id" uuid NOT NULL,
  "user_id" uuid NOT NULL,
  PRIMARY KEY ("bookmark_id", "tag_id"),
  CONSTRAINT "bookmark_tags_bookmark_id_fkey" FOREIGN KEY ("bookmark_id") REFERENCES "public"."bookmarks" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "bookmark_tags_tag_id_fkey" FOREIGN KEY ("tag_id") REFERENCES "public"."tags" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "bookmark_tags_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "collections" table
CREATE TABLE "public"."collections" (
  "id" uuid NOT NULL,
  "title" text NOT NULL,
  "description" text NULL,
  "filter_json" jsonb NOT NULL,
  "user_id" uuid NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "updated_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "collections_user_id_title_key" UNIQUE ("user_id", "title"),
  CONSTRAINT "collections_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "feeds" table
CREATE TABLE "public"."feeds" (
  "id" uuid NOT NULL,
  "source_url" text NOT NULL,
  "link" text NOT NULL,
  "title" text NOT NULL,
  "description" text NULL,
  "refreshed_at" timestamptz NULL,
  "is_custom" boolean NOT NULL DEFAULT false,
  PRIMARY KEY ("id"),
  CONSTRAINT "feeds_source_url_key" UNIQUE ("source_url")
);
-- Create "feed_entries" table
CREATE TABLE "public"."feed_entries" (
  "id" uuid NOT NULL,
  "link" text NOT NULL,
  "title" text NOT NULL,
  "published_at" timestamptz NOT NULL,
  "description" text NULL,
  "author" text NULL,
  "thumbnail_url" text NULL,
  "feed_id" uuid NOT NULL,
  PRIMARY KEY ("id"),
  CONSTRAINT "feed_entries_feed_id_link_key" UNIQUE ("feed_id", "link"),
  CONSTRAINT "feed_entries_feed_id_fkey" FOREIGN KEY ("feed_id") REFERENCES "public"."feeds" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "subscriptions" table
CREATE TABLE "public"."subscriptions" (
  "id" uuid NOT NULL,
  "title" text NOT NULL,
  "description" text NULL,
  "user_id" uuid NOT NULL,
  "feed_id" uuid NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "updated_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "subscriptions_user_id_feed_id_key" UNIQUE ("user_id", "feed_id"),
  CONSTRAINT "subscriptions_feed_id_fkey" FOREIGN KEY ("feed_id") REFERENCES "public"."feeds" ("id") ON UPDATE NO ACTION ON DELETE RESTRICT,
  CONSTRAINT "subscriptions_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "read_entries" table
CREATE TABLE "public"."read_entries" (
  "subscription_id" uuid NOT NULL,
  "feed_entry_id" uuid NOT NULL,
  "user_id" uuid NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("subscription_id", "feed_entry_id"),
  CONSTRAINT "read_entries_feed_entry_id_fkey" FOREIGN KEY ("feed_entry_id") REFERENCES "public"."feed_entries" ("id") ON UPDATE NO ACTION ON DELETE RESTRICT,
  CONSTRAINT "read_entries_subscription_id_fkey" FOREIGN KEY ("subscription_id") REFERENCES "public"."subscriptions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "read_entries_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "streams" table
CREATE TABLE "public"."streams" (
  "id" uuid NOT NULL,
  "title" text NOT NULL,
  "description" text NULL,
  "filter_json" jsonb NOT NULL,
  "user_id" uuid NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "updated_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "streams_user_id_title_key" UNIQUE ("user_id", "title"),
  CONSTRAINT "streams_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "subscription_tags" table
CREATE TABLE "public"."subscription_tags" (
  "subscription_id" uuid NOT NULL,
  "tag_id" uuid NOT NULL,
  "user_id" uuid NOT NULL,
  PRIMARY KEY ("subscription_id", "tag_id"),
  CONSTRAINT "subscription_tags_subscription_id_fkey" FOREIGN KEY ("subscription_id") REFERENCES "public"."subscriptions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "subscription_tags_tag_id_fkey" FOREIGN KEY ("tag_id") REFERENCES "public"."tags" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "subscription_tags_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
