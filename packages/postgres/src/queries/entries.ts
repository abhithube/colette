import { sql } from 'drizzle-orm'
import type { Database } from '../client'
import { entriesTable } from '../schema'
import type { EntryInsert } from '../types'

export async function insertEntry(db: Database, data: EntryInsert) {
	return db
		.insert(entriesTable)
		.values(data)
		.onConflictDoUpdate({
			target: entriesTable.link,
			set: {
				title: sql`excluded.title`,
				publishedAt: sql`excluded.published_at`,
				description: sql`excluded.description`,
				author: sql`excluded.author`,
				thumbnailUrl: sql`excluded.thumbnail_url`,
			},
		})
		.returning({
			id: entriesTable.id,
		})
}
