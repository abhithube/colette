-- Modify "feeds" table
ALTER TABLE "public"."feeds"
DROP CONSTRAINT "feeds_link_key",
DROP COLUMN "xml_url",
ADD COLUMN "source_url" TEXT NOT NULL,
ADD COLUMN "is_custom" BOOLEAN NOT NULL DEFAULT FALSE,
ADD CONSTRAINT "feeds_source_url_key" UNIQUE ("source_url");
