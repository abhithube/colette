-- Create "sessions" table
CREATE TABLE "public"."sessions" (
  "id" TEXT NOT NULL,
  "data" bytea NOT NULL,
  "expiry_date" TIMESTAMPTZ NOT NULL,
  PRIMARY KEY ("id")
);

-- Create "users" table
CREATE TABLE "public"."users" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid (),
  "email" TEXT NOT NULL,
  "password" TEXT NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "users_email_key" UNIQUE ("email")
);

-- Create "folders" table
CREATE TABLE "public"."folders" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid (),
  "title" TEXT NOT NULL,
  "parent_id" uuid NULL,
  "user_id" uuid NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "folders_user_id_parent_id_title_key" UNIQUE ("user_id", "parent_id", "title"),
  CONSTRAINT "folders_parent_id_fkey" FOREIGN KEY ("parent_id") REFERENCES "public"."folders" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "folders_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create "bookmarks" table
CREATE TABLE "public"."bookmarks" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid (),
  "link" TEXT NOT NULL,
  "title" TEXT NOT NULL,
  "thumbnail_url" TEXT NULL,
  "published_at" TIMESTAMPTZ NULL,
  "author" TEXT NULL,
  "folder_id" uuid NULL,
  "user_id" uuid NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "bookmarks_user_id_link_key" UNIQUE ("user_id", "link"),
  CONSTRAINT "bookmarks_folder_id_fkey" FOREIGN KEY ("folder_id") REFERENCES "public"."folders" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "bookmarks_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create "tags" table
CREATE TABLE "public"."tags" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid (),
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
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY ("bookmark_id", "tag_id"),
  CONSTRAINT "bookmark_tags_bookmark_id_fkey" FOREIGN KEY ("bookmark_id") REFERENCES "public"."bookmarks" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "bookmark_tags_tag_id_fkey" FOREIGN KEY ("tag_id") REFERENCES "public"."tags" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "bookmark_tags_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create "feeds" table
CREATE TABLE "public"."feeds" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid (),
  "link" TEXT NOT NULL,
  "title" TEXT NOT NULL,
  "xml_url" TEXT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "feeds_link_key" UNIQUE ("link")
);

-- Create "feed_entries" table
CREATE TABLE "public"."feed_entries" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid (),
  "link" TEXT NOT NULL,
  "title" TEXT NOT NULL,
  "published_at" TIMESTAMPTZ NOT NULL,
  "description" TEXT NULL,
  "author" TEXT NULL,
  "thumbnail_url" TEXT NULL,
  "feed_id" uuid NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "feed_entries_feed_id_link_key" UNIQUE ("feed_id", "link"),
  CONSTRAINT "feed_entries_link_key" UNIQUE ("link"),
  CONSTRAINT "feed_entries_feed_id_fkey" FOREIGN KEY ("feed_id") REFERENCES "public"."feeds" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create "user_feeds" table
CREATE TABLE "public"."user_feeds" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid (),
  "title" TEXT NULL,
  "folder_id" uuid NULL,
  "user_id" uuid NOT NULL,
  "feed_id" uuid NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "user_feeds_user_id_feed_id_key" UNIQUE ("user_id", "feed_id"),
  CONSTRAINT "user_feeds_feed_id_fkey" FOREIGN KEY ("feed_id") REFERENCES "public"."feeds" ("id") ON UPDATE NO ACTION ON DELETE RESTRICT,
  CONSTRAINT "user_feeds_folder_id_fkey" FOREIGN KEY ("folder_id") REFERENCES "public"."folders" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "user_feeds_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create "user_feed_entries" table
CREATE TABLE "public"."user_feed_entries" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid (),
  "has_read" BOOLEAN NOT NULL DEFAULT FALSE,
  "user_feed_id" uuid NOT NULL,
  "feed_entry_id" uuid NOT NULL,
  "user_id" uuid NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "user_feed_entries_user_feed_id_feed_entry_id_key" UNIQUE ("user_feed_id", "feed_entry_id"),
  CONSTRAINT "user_feed_entries_feed_entry_id_fkey" FOREIGN KEY ("feed_entry_id") REFERENCES "public"."feed_entries" ("id") ON UPDATE NO ACTION ON DELETE RESTRICT,
  CONSTRAINT "user_feed_entries_user_feed_id_fkey" FOREIGN KEY ("user_feed_id") REFERENCES "public"."user_feeds" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "user_feed_entries_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Create "user_feed_tags" table
CREATE TABLE "public"."user_feed_tags" (
  "user_feed_id" uuid NOT NULL,
  "tag_id" uuid NOT NULL,
  "user_id" uuid NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY ("user_feed_id", "tag_id"),
  CONSTRAINT "user_feed_tags_tag_id_fkey" FOREIGN KEY ("tag_id") REFERENCES "public"."tags" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "user_feed_tags_user_feed_id_fkey" FOREIGN KEY ("user_feed_id") REFERENCES "public"."user_feeds" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "user_feed_tags_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
