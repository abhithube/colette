import fs from 'node:fs/promises'
import path from 'node:path'
import {
	FileMigrationProvider,
	Kysely,
	Migrator,
	PostgresDialect,
} from 'kysely'
import { Pool } from 'pg'

async function migrateToLatest() {
	const db = new Kysely<any>({
		dialect: new PostgresDialect({
			pool: new Pool({
				connectionString: process.env.DATABASE_URL,
			}),
		}),
	})

	const migrator = new Migrator({
		db,
		provider: new FileMigrationProvider({
			fs,
			path,
			migrationFolder: path.join(__dirname, '..', '/migrations'),
		}),
	})

	const { error, results } = await migrator.migrateToLatest()
	if (error || !results) {
		console.error('failed to migrate')
		console.error(error)
		process.exit(1)
	}

	for (const it of results) {
		if (it.status === 'Success') {
			console.log(`migration "${it.migrationName}" was executed successfully`)
		} else if (it.status === 'Error') {
			console.error(`failed to execute migration "${it.migrationName}"`)
		}
	}

	await db.destroy()
}

migrateToLatest()
