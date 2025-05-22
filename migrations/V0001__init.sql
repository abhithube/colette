-- Create "jobs" table
CREATE TABLE "public"."jobs" (
  "id" uuid NOT NULL,
  "job_type" TEXT NOT NULL,
  "data_json" JSONB NOT NULL,
  "status" TEXT NOT NULL DEFAULT 'pending',
  "group_identifier" TEXT NULL,
  "message" TEXT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "completed_at" TIMESTAMPTZ NULL,
  PRIMARY KEY ("id")
);

-- Create "users" table
CREATE TABLE "public"."users" (
  "id" uuid NOT NULL,
  "external_id" TEXT NOT NULL,
  "email" TEXT NULL,
  "display_name" TEXT NULL,
  "picture_url" TEXT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "users_external_id_key" UNIQUE ("external_id")
);

-- Create "api_keys" table
CREATE TABLE "public"."api_keys" (
  "id" uuid NOT NULL,
  "lookup_hash" TEXT NOT NULL,
  "verification_hash" TEXT NOT NULL,
  "title" TEXT NOT NULL,
  "preview" TEXT NOT NULL,
  "user_id" uuid NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "api_keys_lookup_hash_key" UNIQUE ("lookup_hash"),
  CONSTRAINT "api_keys_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create "bookmarks" table
CREATE TABLE "public"."bookmarks" (
  "id" uuid NOT NULL,
  "link" TEXT NOT NULL,
  "title" TEXT NOT NULL,
  "thumbnail_url" TEXT NULL,
  "published_at" TIMESTAMPTZ NULL,
  "author" TEXT NULL,
  "archived_path" TEXT NULL,
  "user_id" uuid NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "bookmarks_user_id_link_key" UNIQUE ("user_id", "link"),
  CONSTRAINT "bookmarks_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create "tags" table
CREATE TABLE "public"."tags" (
  "id" uuid NOT NULL,
  "title" TEXT NOT NULL,
  "user_id" uuid NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
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
  "title" TEXT NOT NULL,
  "description" TEXT NULL,
  "filter_json" JSONB NOT NULL,
  "user_id" uuid NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "collections_user_id_title_key" UNIQUE ("user_id", "title"),
  CONSTRAINT "collections_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create "feeds" table
CREATE TABLE "public"."feeds" (
  "id" uuid NOT NULL,
  "source_url" TEXT NOT NULL,
  "link" TEXT NOT NULL,
  "title" TEXT NOT NULL,
  "description" TEXT NULL,
  "refreshed_at" TIMESTAMPTZ NULL,
  "is_custom" BOOLEAN NOT NULL DEFAULT FALSE,
  PRIMARY KEY ("id"),
  CONSTRAINT "feeds_source_url_key" UNIQUE ("source_url")
);

-- Create "feed_entries" table
CREATE TABLE "public"."feed_entries" (
  "id" uuid NOT NULL,
  "link" TEXT NOT NULL,
  "title" TEXT NOT NULL,
  "published_at" TIMESTAMPTZ NOT NULL,
  "description" TEXT NULL,
  "author" TEXT NULL,
  "thumbnail_url" TEXT NULL,
  "feed_id" uuid NOT NULL,
  PRIMARY KEY ("id"),
  CONSTRAINT "feed_entries_feed_id_link_key" UNIQUE ("feed_id", "link"),
  CONSTRAINT "feed_entries_feed_id_fkey" FOREIGN KEY ("feed_id") REFERENCES "public"."feeds" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create "subscriptions" table
CREATE TABLE "public"."subscriptions" (
  "id" uuid NOT NULL,
  "title" TEXT NOT NULL,
  "description" TEXT NULL,
  "user_id" uuid NOT NULL,
  "feed_id" uuid NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
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
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY ("subscription_id", "feed_entry_id"),
  CONSTRAINT "read_entries_feed_entry_id_fkey" FOREIGN KEY ("feed_entry_id") REFERENCES "public"."feed_entries" ("id") ON UPDATE NO ACTION ON DELETE RESTRICT,
  CONSTRAINT "read_entries_subscription_id_fkey" FOREIGN KEY ("subscription_id") REFERENCES "public"."subscriptions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "read_entries_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create "streams" table
CREATE TABLE "public"."streams" (
  "id" uuid NOT NULL,
  "title" TEXT NOT NULL,
  "description" TEXT NULL,
  "filter_json" JSONB NOT NULL,
  "user_id" uuid NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
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
