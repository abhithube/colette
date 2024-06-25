import { and, asc, count, eq } from 'drizzle-orm'
import type { Database } from '../client'
import {
	feedEntriesTable,
	feedsTable,
	profileFeedEntriesTable,
	profileFeedsTable,
} from '../schema'
import type { ProfileFeedInsert, SelectWithProfileParams } from '../types'

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

export type ProfileFeedSelectParams = {
	profileId: string
}

export async function selectProfileFeeds(
	db: Database,
	params: ProfileFeedSelectParams,
) {
	return db
		.select(columns)
		.from(profileFeedsTable)
		.innerJoin(feedsTable, eq(feedsTable.id, profileFeedsTable.feedId))
		.leftJoin(
			feedEntriesTable,
			eq(feedEntriesTable.feedId, profileFeedsTable.feedId),
		)
		.leftJoin(
			profileFeedEntriesTable,
			and(
				eq(profileFeedEntriesTable.feedEntryId, feedEntriesTable.id),
				eq(profileFeedEntriesTable.hasRead, false),
			),
		)
		.where(eq(profileFeedsTable.profileId, params.profileId))
		.orderBy(asc(profileFeedsTable.customTitle), asc(feedsTable.title))
}

export async function selectProfileFeedById(
	db: Database,
	params: SelectWithProfileParams,
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

export async function deleteProfileFeed(
	db: Database,
	params: SelectWithProfileParams,
) {
	return db
		.delete(profileFeedsTable)
		.where(
			and(
				eq(profileFeedsTable.id, params.id),
				eq(profileFeedsTable.profileId, params.profileId),
			),
		)
}
