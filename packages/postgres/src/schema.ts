import {
	integer,
	pgTable,
	serial,
	text,
	timestamp,
	unique,
} from 'drizzle-orm/pg-core'

export const feedsTable = pgTable('feeds', {
	id: serial('id').primaryKey(),
	link: text('link').notNull().unique(),
	title: text('title').notNull(),
	url: text('url'),
})

export type FeedInsert = typeof feedsTable.$inferInsert

export const entriesTable = pgTable('entries', {
	id: serial('id').primaryKey(),
	link: text('link').notNull().unique(),
	title: text('title').notNull(),
	publishedAt: timestamp('published_at', { withTimezone: true }),
	description: text('description'),
	author: text('author'),
	thumbnailUrl: text('thumbnail_url'),
})

export type EntryInsert = typeof entriesTable.$inferInsert

export const feedEntriesTable = pgTable(
	'feed_entries',
	{
		id: serial('id').primaryKey(),
		feedId: integer('feed_id')
			.notNull()
			.references(() => feedsTable.id, { onDelete: 'cascade' }),
		entryId: integer('entry_id')
			.notNull()
			.references(() => entriesTable.id, { onDelete: 'cascade' }),
	},
	(t) => ({
		feedId_EntryIdUnq: unique().on(t.feedId, t.entryId),
	}),
)

export type FeedEntryInsert = typeof feedEntriesTable.$inferInsert
