import { pgTable, serial, text } from 'drizzle-orm/pg-core'

export const feedsTable = pgTable('feeds', {
	id: serial('id').primaryKey(),
	link: text('link').notNull().unique(),
	title: text('title').notNull(),
	url: text('url'),
})
