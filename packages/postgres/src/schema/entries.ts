import { pgTable, serial, text, timestamp } from 'drizzle-orm/pg-core'

export const entriesTable = pgTable('entries', {
	id: serial('id').primaryKey(),
	link: text('link').notNull().unique(),
	title: text('title').notNull(),
	publishedAt: timestamp('published_at', { withTimezone: true }),
	description: text('description'),
	author: text('author'),
	thumbnailUrl: text('thumbnail_url'),
})
