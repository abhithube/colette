import { pgTable, text, timestamp } from 'drizzle-orm/pg-core'
import { profilesTable } from './profiles'
import { usersTable } from './users'

export const sessionsTable = pgTable('sessions', {
	id: text('id').primaryKey(),
	expiresAt: timestamp('expires_at', { withTimezone: true }).notNull(),
	userId: text('user_id')
		.notNull()
		.references(() => usersTable.id, { onDelete: 'cascade' }),
	profileId: text('profile_id')
		.notNull()
		.references(() => profilesTable.id, { onDelete: 'cascade' }),
})
