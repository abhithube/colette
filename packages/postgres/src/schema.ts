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

export const entriesTable = pgTable('entries', {
	id: serial('id').primaryKey(),
	link: text('link').notNull().unique(),
	title: text('title').notNull(),
	publishedAt: timestamp('published_at', { withTimezone: true }),
	description: text('description'),
	author: text('author'),
	thumbnailUrl: text('thumbnail_url'),
})

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

export const usersTable = pgTable('users', {
	id: text('id').primaryKey(),
	email: text('email').notNull().unique(),
	password: text('password').notNull(),
	createdAt: timestamp('created_at', { withTimezone: true })
		.notNull()
		.defaultNow(),
	updatedAt: timestamp('updated_at', { withTimezone: true })
		.notNull()
		.defaultNow(),
})

export const sessionsTable = pgTable('sessions', {
	id: text('id').primaryKey(),
	expiresAt: timestamp('expires_at', { withTimezone: true }).notNull(),
	userId: text('user_id')
		.notNull()
		.references(() => usersTable.id, { onDelete: 'cascade' }),
})
