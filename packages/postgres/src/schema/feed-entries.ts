import { integer, pgTable, serial, unique } from 'drizzle-orm/pg-core'
import { entriesTable } from './entries'
import { feedsTable } from './feeds'

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
		feedIdEntryIdUnq: unique().on(t.feedId, t.entryId),
	}),
)
