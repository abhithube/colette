import { integer, pgTable, text, timestamp, unique } from 'drizzle-orm/pg-core'
import { feedsTable } from './feeds'
import { profilesTable } from './profiles'

export const profileFeedsTable = pgTable(
	'profile_feeds',
	{
		id: text('id').primaryKey(),
		customTitle: text('custom_title'),
		profileId: text('profile_id')
			.notNull()
			.references(() => profilesTable.id, { onDelete: 'cascade' }),
		feedId: integer('feed_id')
			.notNull()
			.references(() => feedsTable.id, { onDelete: 'restrict' }),
		createdAt: timestamp('created_at', { mode: 'string', withTimezone: true })
			.notNull()
			.defaultNow(),
		updatedAt: timestamp('updated_at', { mode: 'string', withTimezone: true })
			.notNull()
			.defaultNow(),
	},
	(t) => ({
		profileIdFeedIdUnq: unique().on(t.profileId, t.feedId),
	}),
)
