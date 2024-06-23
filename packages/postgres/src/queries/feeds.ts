import { sql } from 'drizzle-orm'
import type { Database } from '../client'
import { feedsTable } from '../schema'
import type { FeedInsert } from '../types'

export async function insertFeed(db: Database, data: FeedInsert) {
	return db
		.insert(feedsTable)
		.values(data)
		.onConflictDoUpdate({
			target: feedsTable.link,
			set: {
				title: sql`excluded.title`,
			},
		})
		.returning({
			id: feedsTable.id,
		})
}
