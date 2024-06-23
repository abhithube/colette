import type { Insertable } from 'kysely'
import type { Users } from 'kysely-codegen'
import type { Database } from './client'

const columns = ['id', 'email', 'password', 'created_at', 'updated_at'] as const

export async function findByEmail(db: Database, params: Pick<Users, 'email'>) {
	return db
		.selectFrom('users')
		.select(columns)
		.where('email', '=', params.email)
		.executeTakeFirst()
}

export async function create(db: Database, data: Insertable<Users>) {
	return db
		.insertInto('users')
		.values(data)
		.returning(columns)
		.executeTakeFirstOrThrow()
}
