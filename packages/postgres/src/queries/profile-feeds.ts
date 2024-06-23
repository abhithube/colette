import { and, count, eq } from 'drizzle-orm'
import type { Database } from '../client'
import {
	feedEntriesTable,
	feedsTable,
	profileFeedEntriesTable,
	profileFeedsTable,
} from '../schema'
import type { ProfileFeedInsert, SelectParams } from '../types'

const columns = {
	id: profileFeedsTable.id,
	link: feedsTable.link,
	title: feedsTable.title,
	url: feedsTable.url,
	customTitle: profileFeedsTable.customTitle,
	feedId: profileFeedsTable.feedId,
	createdAt: profileFeedsTable.createdAt,
	updatedAt: profileFeedsTable.updatedAt,
	unreadCount: count(profileFeedEntriesTable.id),
}

export async function selectProfileFeedById(
	db: Database,
	params: SelectParams,
) {
	return db
		.select(columns)
		.from(profileFeedsTable)
		.innerJoin(feedsTable, eq(feedsTable.id, profileFeedsTable.feedId))
		.innerJoin(feedEntriesTable, eq(feedEntriesTable.feedId, feedsTable.id))
		.innerJoin(
			profileFeedEntriesTable,
			eq(profileFeedEntriesTable.feedEntryId, feedEntriesTable.id),
		)
		.where(
			and(
				eq(profileFeedsTable.id, params.id),
				eq(profileFeedsTable.profileId, params.profileId),
			),
		)
}

export async function insertProfileFeed(db: Database, data: ProfileFeedInsert) {
	return db
		.insert(profileFeedsTable)
		.values(data)
		.onConflictDoNothing({
			target: [profileFeedsTable.profileId, profileFeedsTable.feedId],
		})
		.returning({
			id: profileFeedsTable.id,
		})
}
