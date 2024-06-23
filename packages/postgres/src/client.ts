import { drizzle } from 'drizzle-orm/node-postgres'
import { Pool } from 'pg'
import {
	entriesTable,
	feedEntriesTable,
	feedsTable,
	profilesTable,
	usersTable,
} from './schema'

const schema = {
	entries: entriesTable,
	feedEntries: feedEntriesTable,
	feeds: feedsTable,
	users: usersTable,
	profiles: profilesTable,
}

export function createPostgresClient(connectionString: string) {
	const client = new Pool({
		connectionString,
	})

	return drizzle(client, {
		schema,
	})
}

export type Database = ReturnType<typeof createPostgresClient>
