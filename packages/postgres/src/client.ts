import { drizzle } from 'drizzle-orm/node-postgres'
import type { Client, Pool } from 'pg'
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

export function createDatabase(client: Client | Pool) {
	return drizzle(client, {
		schema,
	})
}

export type Database = ReturnType<typeof createDatabase>
