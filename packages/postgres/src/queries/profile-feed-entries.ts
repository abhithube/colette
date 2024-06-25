import { and, eq } from 'drizzle-orm'
import type { Database } from '../client'
import {
	entriesTable,
	feedEntriesTable,
	profileFeedEntriesTable,
	profileFeedsTable,
} from '../schema'
import type {
	ProfileFeedEntryDeleteParams,
	ProfileFeedEntryInsert,
	SelectWithProfileParams,
} from '../types'

const columns = {
	id: profileFeedEntriesTable.id,
	link: entriesTable.link,
	title: entriesTable.title,
	publishedAt: entriesTable.publishedAt,
	description: entriesTable.description,
	author: entriesTable.author,
	thumbnailUrl: entriesTable.thumbnailUrl,
	hasRead: profileFeedEntriesTable.hasRead,
	feedId: profileFeedEntriesTable.profileFeedId,
}

export async function selectProfileFeedEntryById(
	db: Database,
	params: SelectWithProfileParams,
) {
	return db
		.select(columns)
		.from(profileFeedEntriesTable)
		.innerJoin(
			profileFeedsTable,
			eq(profileFeedsTable.id, profileFeedEntriesTable.profileFeedId),
		)
		.innerJoin(
			feedEntriesTable,
			eq(feedEntriesTable.id, profileFeedEntriesTable.feedEntryId),
		)
		.innerJoin(entriesTable, eq(entriesTable.id, feedEntriesTable.entryId))
		.where(
			and(
				eq(profileFeedEntriesTable.id, params.id),
				eq(profileFeedsTable.profileId, params.profileId),
			),
		)
}

export async function insertProfileFeedEntry(
	db: Database,
	data: ProfileFeedEntryInsert,
) {
	return db
		.insert(profileFeedEntriesTable)
		.values(data)
		.onConflictDoNothing({
			target: [
				profileFeedEntriesTable.profileFeedId,
				profileFeedEntriesTable.feedEntryId,
			],
		})
		.returning({
			id: profileFeedEntriesTable.id,
		})
}

export async function deleteProfileFeedEntries(
	db: Database,
	params: ProfileFeedEntryDeleteParams,
) {
	return db
		.delete(profileFeedEntriesTable)
		.where(eq(profileFeedEntriesTable.profileFeedId, params.profileFeedId))
}
