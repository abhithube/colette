import { boolean, pgTable, text, timestamp, unique } from 'drizzle-orm/pg-core'
import { usersTable } from './users'

export const profilesTable = pgTable(
	'profiles',
	{
		id: text('id').primaryKey(),
		title: text('title').notNull(),
		imageUrl: text('image_url'),
		userId: text('user_id')
			.notNull()
			.references(() => usersTable.id, {
				onDelete: 'cascade',
			}),
		isDefault: boolean('is_default').notNull().default(false),
		createdAt: timestamp('created_at', { mode: 'string', withTimezone: true })
			.notNull()
			.defaultNow(),
		updatedAt: timestamp('updated_at', { mode: 'string', withTimezone: true })
			.notNull()
			.defaultNow(),
	},
	(t) => ({
		userIdIsDefaultUnq: unique().on(t.userId, t.isDefault),
	}),
)
