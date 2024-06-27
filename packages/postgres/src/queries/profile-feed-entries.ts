import { and, asc, desc, eq, lt } from 'drizzle-orm'
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

export type ProfileFeedEntrySelectParams = {
	profileId: string
	publishedAt?: string
	profileFeedId?: string
}

export async function selectProfileFeedEntries(
	db: Database,
	params: ProfileFeedEntrySelectParams,
) {
	let query = db
		.select(columns)
		.from(profileFeedEntriesTable)
		.innerJoin(
			feedEntriesTable,
			eq(feedEntriesTable.id, profileFeedEntriesTable.feedEntryId),
		)
		.innerJoin(entriesTable, eq(entriesTable.id, feedEntriesTable.entryId))
		.limit(25)
		.orderBy(desc(entriesTable.publishedAt), asc(entriesTable.title))
		.$dynamic()

	const publishedAtFilter = params.publishedAt
		? lt(entriesTable.publishedAt, params.publishedAt)
		: undefined

	if (params.profileFeedId) {
		query = query
			.innerJoin(
				profileFeedsTable,
				eq(profileFeedsTable.feedId, feedEntriesTable.feedId),
			)
			.where(
				and(eq(profileFeedsTable.id, params.profileFeedId), publishedAtFilter),
			)
	} else {
		query = query.where(publishedAtFilter)
	}

	return query
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
