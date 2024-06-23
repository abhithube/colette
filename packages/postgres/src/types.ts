import type {
	entriesTable,
	feedEntriesTable,
	feedsTable,
	usersTable,
} from './schema'

export type EntryInsert = typeof entriesTable.$inferInsert

export type FeedInsert = typeof feedsTable.$inferInsert

export type FeedEntryInsert = typeof feedEntriesTable.$inferInsert

export type UserInsert = typeof usersTable.$inferInsert
