import type {
	entriesTable,
	feedEntriesTable,
	feedsTable,
	profileFeedEntriesTable,
	profileFeedsTable,
	profilesTable,
	usersTable,
} from './schema'

export type SelectParams = {
	id: string
	profileId: string
}

export type EntryInsert = typeof entriesTable.$inferInsert

export type FeedInsert = typeof feedsTable.$inferInsert

export type FeedEntryInsert = typeof feedEntriesTable.$inferInsert

export type ProfileInsert = typeof profilesTable.$inferInsert
export type ProfileSelectParams = Pick<
	typeof profilesTable.$inferSelect,
	'userId'
>
export type ProfileSelectByIdParams = Pick<
	typeof profilesTable.$inferSelect,
	'id' | 'userId'
>

export type ProfileFeedInsert = typeof profileFeedsTable.$inferInsert

export type ProfileFeedEntryInsert = typeof profileFeedEntriesTable.$inferInsert

export type UserInsert = typeof usersTable.$inferInsert
export type UserSelectByEmailParams = Pick<
	typeof usersTable.$inferSelect,
	'email'
>
