-- Create "feeds" table
CREATE TABLE "public"."feeds" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "link" text NOT NULL,
    "title" text NOT NULL,
    "xml_url" text NULL,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "updated_at" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("id"),
    CONSTRAINT "feeds_link_key" UNIQUE ("link")
);
-- Create "sessions" table
CREATE TABLE "public"."sessions" (
    "id" text NOT NULL,
    "data" bytea NOT NULL,
    "expiry_date" timestamptz NOT NULL,
    PRIMARY KEY ("id")
);
-- Create "feed_entries" table
CREATE TABLE "public"."feed_entries" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "link" text NOT NULL,
    "title" text NOT NULL,
    "published_at" timestamptz NOT NULL,
    "description" text NULL,
    "author" text NULL,
    "thumbnail_url" text NULL,
    "feed_id" uuid NOT NULL,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "updated_at" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("id"),
    CONSTRAINT "feed_entries_feed_id_link_key" UNIQUE ("feed_id", "link"),
    CONSTRAINT "feed_entries_link_key" UNIQUE ("link"),
    CONSTRAINT "feed_entries_feed_id_fkey" FOREIGN KEY (
        "feed_id"
    ) REFERENCES "public"."feeds" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "users" table
CREATE TABLE "public"."users" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "email" text NOT NULL,
    "password" text NOT NULL,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "updated_at" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("id"),
    CONSTRAINT "users_email_key" UNIQUE ("email")
);
-- Create "folders" table
CREATE TABLE "public"."folders" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "title" text NOT NULL,
    "parent_id" uuid NULL,
    "user_id" uuid NOT NULL,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "updated_at" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("id"),
    CONSTRAINT "folders_user_id_parent_id_title_key" UNIQUE (
        "user_id", "parent_id", "title"
    ),
    CONSTRAINT "folders_parent_id_fkey" FOREIGN KEY (
        "parent_id"
    ) REFERENCES "public"."folders" (
        "id"
    ) ON UPDATE NO ACTION ON DELETE CASCADE,
    CONSTRAINT "folders_user_id_fkey" FOREIGN KEY (
        "user_id"
    ) REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "tags" table
CREATE TABLE "public"."tags" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "title" text NOT NULL,
    "user_id" uuid NOT NULL,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "updated_at" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("id"),
    CONSTRAINT "tags_user_id_title_key" UNIQUE ("user_id", "title"),
    CONSTRAINT "tags_user_id_fkey" FOREIGN KEY (
        "user_id"
    ) REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "bookmarks" table
CREATE TABLE "public"."bookmarks" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "link" text NOT NULL,
    "title" text NOT NULL,
    "thumbnail_url" text NULL,
    "published_at" timestamptz NULL,
    "author" text NULL,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "updated_at" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("id"),
    CONSTRAINT "bookmarks_link_key" UNIQUE ("link")
);
-- Create "user_bookmarks" table
CREATE TABLE "public"."user_bookmarks" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "title" text NULL,
    "thumbnail_url" text NULL,
    "published_at" timestamptz NULL,
    "author" text NULL,
    "folder_id" uuid NULL,
    "user_id" uuid NOT NULL,
    "bookmark_id" uuid NOT NULL,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "updated_at" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("id"),
    CONSTRAINT "user_bookmarks_user_id_bookmark_id_key" UNIQUE (
        "user_id", "bookmark_id"
    ),
    CONSTRAINT "user_bookmarks_bookmark_id_fkey" FOREIGN KEY (
        "bookmark_id"
    ) REFERENCES "public"."bookmarks" (
        "id"
    ) ON UPDATE NO ACTION ON DELETE RESTRICT,
    CONSTRAINT "user_bookmarks_folder_id_fkey" FOREIGN KEY (
        "folder_id"
    ) REFERENCES "public"."folders" (
        "id"
    ) ON UPDATE NO ACTION ON DELETE CASCADE,
    CONSTRAINT "user_bookmarks_user_id_fkey" FOREIGN KEY (
        "user_id"
    ) REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "user_bookmark_tags" table
CREATE TABLE "public"."user_bookmark_tags" (
    "user_bookmark_id" uuid NOT NULL,
    "tag_id" uuid NOT NULL,
    "user_id" uuid NOT NULL,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "updated_at" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("user_bookmark_id", "tag_id"),
    CONSTRAINT "user_bookmark_tags_tag_id_fkey" FOREIGN KEY (
        "tag_id"
    ) REFERENCES "public"."tags" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
    CONSTRAINT "user_bookmark_tags_user_bookmark_id_fkey" FOREIGN KEY (
        "user_bookmark_id"
    ) REFERENCES "public"."user_bookmarks" (
        "id"
    ) ON UPDATE NO ACTION ON DELETE CASCADE,
    CONSTRAINT "user_bookmark_tags_user_id_fkey" FOREIGN KEY (
        "user_id"
    ) REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "user_feeds" table
CREATE TABLE "public"."user_feeds" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "title" text NULL,
    "folder_id" uuid NULL,
    "user_id" uuid NOT NULL,
    "feed_id" uuid NOT NULL,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "updated_at" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("id"),
    CONSTRAINT "user_feeds_user_id_feed_id_key" UNIQUE ("user_id", "feed_id"),
    CONSTRAINT "user_feeds_feed_id_fkey" FOREIGN KEY (
        "feed_id"
    ) REFERENCES "public"."feeds" ("id") ON UPDATE NO ACTION ON DELETE RESTRICT,
    CONSTRAINT "user_feeds_folder_id_fkey" FOREIGN KEY (
        "folder_id"
    ) REFERENCES "public"."folders" (
        "id"
    ) ON UPDATE NO ACTION ON DELETE CASCADE,
    CONSTRAINT "user_feeds_user_id_fkey" FOREIGN KEY (
        "user_id"
    ) REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "user_feed_entries" table
CREATE TABLE "public"."user_feed_entries" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "has_read" boolean NOT NULL DEFAULT false,
    "user_feed_id" uuid NOT NULL,
    "feed_entry_id" uuid NOT NULL,
    "user_id" uuid NOT NULL,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "updated_at" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("id"),
    CONSTRAINT "user_feed_entries_user_feed_id_feed_entry_id_key" UNIQUE (
        "user_feed_id", "feed_entry_id"
    ),
    CONSTRAINT "user_feed_entries_feed_entry_id_fkey" FOREIGN KEY (
        "feed_entry_id"
    ) REFERENCES "public"."feed_entries" (
        "id"
    ) ON UPDATE NO ACTION ON DELETE RESTRICT,
    CONSTRAINT "user_feed_entries_user_feed_id_fkey" FOREIGN KEY (
        "user_feed_id"
    ) REFERENCES "public"."user_feeds" (
        "id"
    ) ON UPDATE NO ACTION ON DELETE CASCADE,
    CONSTRAINT "user_feed_entries_user_id_fkey" FOREIGN KEY (
        "user_id"
    ) REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "user_feed_tags" table
CREATE TABLE "public"."user_feed_tags" (
    "user_feed_id" uuid NOT NULL,
    "tag_id" uuid NOT NULL,
    "user_id" uuid NOT NULL,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "updated_at" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("user_feed_id", "tag_id"),
    CONSTRAINT "user_feed_tags_tag_id_fkey" FOREIGN KEY (
        "tag_id"
    ) REFERENCES "public"."tags" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
    CONSTRAINT "user_feed_tags_user_feed_id_fkey" FOREIGN KEY (
        "user_feed_id"
    ) REFERENCES "public"."user_feeds" (
        "id"
    ) ON UPDATE NO ACTION ON DELETE CASCADE,
    CONSTRAINT "user_feed_tags_user_id_fkey" FOREIGN KEY (
        "user_id"
    ) REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
