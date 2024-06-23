import type { Database } from '../client'
import { type FeedEntryInsert, feedEntriesTable } from '../schema'

export async function insertFeedEntry(db: Database, data: FeedEntryInsert) {
	return db
		.insert(feedEntriesTable)
		.values(data)
		.onConflictDoNothing({
			target: [feedEntriesTable.feedId, feedEntriesTable.entryId],
		})
		.returning({
			id: feedEntriesTable.id,
		})
}
