import type { Insertable } from 'kysely'
import type { Entries } from 'kysely-codegen'
import type { Database } from './client'

export async function create(db: Database, data: Insertable<Entries>) {
	return db
		.insertInto('entries')
		.values(data)
		.onConflict((oc) =>
			oc.column('link').doUpdateSet({
				title: (eb) => eb.ref('excluded.title'),
				published_at: (eb) => eb.ref('excluded.published_at'),
				description: (eb) => eb.ref('excluded.description'),
				author: (eb) => eb.ref('excluded.author'),
				thumbnail_url: (eb) => eb.ref('excluded.thumbnail_url'),
			}),
		)
		.returning('id')
		.executeTakeFirstOrThrow()
}
