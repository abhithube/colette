CREATE TABLE IF NOT EXISTS "profile_feed_entries" (
	"id" text PRIMARY KEY NOT NULL,
	"has_read" boolean DEFAULT false NOT NULL,
	"profile_feed_id" text NOT NULL,
	"feed_entry_id" integer NOT NULL,
	CONSTRAINT "profile_feed_entries_profile_feed_id_feed_entry_id_unique" UNIQUE("profile_feed_id","feed_entry_id")
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "profile_feeds" (
	"id" text PRIMARY KEY NOT NULL,
	"custom_title" text,
	"profile_id" text NOT NULL,
	"feed_id" integer NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	"updated_at" timestamp with time zone DEFAULT now() NOT NULL,
	CONSTRAINT "profile_feeds_profile_id_feed_id_unique" UNIQUE("profile_id","feed_id")
);
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "profile_feed_entries" ADD CONSTRAINT "profile_feed_entries_profile_feed_id_profile_feeds_id_fk" FOREIGN KEY ("profile_feed_id") REFERENCES "public"."profile_feeds"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "profile_feed_entries" ADD CONSTRAINT "profile_feed_entries_feed_entry_id_feed_entries_id_fk" FOREIGN KEY ("feed_entry_id") REFERENCES "public"."feed_entries"("id") ON DELETE restrict ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "profile_feeds" ADD CONSTRAINT "profile_feeds_profile_id_profiles_id_fk" FOREIGN KEY ("profile_id") REFERENCES "public"."profiles"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "profile_feeds" ADD CONSTRAINT "profile_feeds_feed_id_feeds_id_fk" FOREIGN KEY ("feed_id") REFERENCES "public"."feeds"("id") ON DELETE restrict ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
