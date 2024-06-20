import type { Insertable } from 'kysely'
import type { Feeds } from 'kysely-codegen'
import type { Database } from './client'

export async function create(db: Database, data: Insertable<Feeds>) {
	return db
		.insertInto('feeds')
		.values(data)
		.onConflict((oc) =>
			oc.column('link').doUpdateSet({
				title: (eb) => eb.ref('excluded.title'),
				url: (eb) => eb.ref('excluded.url'),
			}),
		)
		.returning('id')
		.executeTakeFirstOrThrow()
}
