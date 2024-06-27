import { boolean, integer, pgTable, text, unique } from 'drizzle-orm/pg-core'
import { feedEntriesTable } from './feed-entries'
import { profileFeedsTable } from './profile-feeds'

export const profileFeedEntriesTable = pgTable(
	'profile_feed_entries',
	{
		id: text('id').primaryKey(),
		hasRead: boolean('has_read').notNull().default(false),
		profileFeedId: text('profile_feed_id')
			.notNull()
			.references(() => profileFeedsTable.id, { onDelete: 'cascade' }),
		feedEntryId: integer('feed_entry_id')
			.notNull()
			.references(() => feedEntriesTable.id, { onDelete: 'restrict' }),
	},
	(t) => ({
		profileFeedIdFeedEntryIdUnq: unique().on(t.profileFeedId, t.feedEntryId),
	}),
)
