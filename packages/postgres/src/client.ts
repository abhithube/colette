import { drizzle } from 'drizzle-orm/node-postgres'
import { Pool } from 'pg'
import {
	entriesTable,
	feedEntriesTable,
	feedsTable,
	profileFeedEntriesTable,
	profileFeedsTable,
	profilesTable,
	usersTable,
} from './schema'

const schema = {
	entries: entriesTable,
	feedEntries: feedEntriesTable,
	feeds: feedsTable,
	profileFeedEntries: profileFeedEntriesTable,
	profileFeeds: profileFeedsTable,
	profiles: profilesTable,
	users: usersTable,
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
