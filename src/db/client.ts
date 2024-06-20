import {
	Kysely,
	PostgresDialect,
	type QueryCreator,
	type SelectQueryBuilder,
} from 'kysely'
import type { DB } from 'kysely-codegen'
import { Pool } from 'pg'

export const pool = new Pool({
	connectionString: process.env.DATABASE_URL,
})

const dialect = new PostgresDialect({
	pool,
})

export const db = new Kysely<DB>({
	dialect,
})

export type Database = QueryCreator<DB>

export type Select<TB extends keyof DB> = Parameters<
	SelectQueryBuilder<DB, TB, unknown>['select']
>[0]
