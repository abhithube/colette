import type { Insertable } from 'kysely'
import type { FeedEntries } from 'kysely-codegen'
import type { Database } from './client'

export async function create(db: Database, data: Insertable<FeedEntries>) {
	return db
		.with('fe', (db) =>
			db
				.insertInto('feed_entries')
				.values(data)
				.onConflict((oc) => oc.columns(['feed_id', 'entry_id']).doNothing())
				.returning('id'),
		)
		.selectFrom('fe')
		.select('id')
		.unionAll(
			db
				.selectFrom('feed_entries')
				.select('id')
				.where('feed_id', '=', data.feed_id)
				.where('entry_id', '=', data.entry_id),
		)
		.executeTakeFirstOrThrow()
}
