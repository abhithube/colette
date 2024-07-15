import { pgTable, text, timestamp } from 'drizzle-orm/pg-core'

export const usersTable = pgTable('users', {
	id: text('id').primaryKey(),
	email: text('email').notNull().unique(),
	password: text('password').notNull(),
	createdAt: timestamp('created_at', { mode: 'string', withTimezone: true })
		.notNull()
		.defaultNow(),
	updatedAt: timestamp('updated_at', { mode: 'string', withTimezone: true })
		.notNull()
		.defaultNow(),
})
