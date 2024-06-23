import type {
	entriesTable,
	feedEntriesTable,
	feedsTable,
	profilesTable,
	usersTable,
} from './schema'

export type EntryInsert = typeof entriesTable.$inferInsert

export type FeedInsert = typeof feedsTable.$inferInsert

export type FeedEntryInsert = typeof feedEntriesTable.$inferInsert

export type ProfileInsert = typeof profilesTable.$inferInsert

export type UserInsert = typeof usersTable.$inferInsert
export type UserSelectByEmailParams = Pick<
	typeof usersTable.$inferSelect,
	'email'
>
