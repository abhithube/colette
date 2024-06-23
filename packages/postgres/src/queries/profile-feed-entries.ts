import { and, eq } from 'drizzle-orm'
import type { Database } from '../client'
import {
	entriesTable,
	feedEntriesTable,
	profileFeedEntriesTable,
	profileFeedsTable,
} from '../schema'
import type { ProfileFeedEntryInsert, SelectParams } from '../types'

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

export async function selectProfileFeedById(
	db: Database,
	params: SelectParams,
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
