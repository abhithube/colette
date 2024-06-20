import { Kysely, PostgresDialect, type QueryCreator } from 'kysely'
import type { DB } from 'kysely-codegen'
import { Pool } from 'pg'

const dialect = new PostgresDialect({
	pool: new Pool({
		connectionString: process.env.DATABASE_URL,
	}),
})

export const db = new Kysely<DB>({
	dialect,
})

export type Database = QueryCreator<DB>
