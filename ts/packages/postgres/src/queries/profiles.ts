import { and, eq } from 'drizzle-orm'
import type { Database } from '../client'
import { profilesTable } from '../schema'
import type {
	ProfileInsert,
	ProfileSelectByIdParams,
	ProfileSelectParams,
} from '../types'

const columns = {
	id: profilesTable.id,
	title: profilesTable.title,
	imageUrl: profilesTable.imageUrl,
	userId: profilesTable.userId,
	createdAt: profilesTable.createdAt,
	updatedAt: profilesTable.updatedAt,
}

export async function selectProfiles(
	db: Database,
	params: ProfileSelectParams,
) {
	return db
		.select(columns)
		.from(profilesTable)
		.where(eq(profilesTable.userId, params.userId))
}

export async function selectProfileById(
	db: Database,
	params: ProfileSelectByIdParams,
) {
	return db
		.select(columns)
		.from(profilesTable)
		.where(
			and(
				eq(profilesTable.id, params.id),
				eq(profilesTable.userId, params.userId),
			),
		)
}

export async function selectDefaultProfile(
	db: Database,
	params: ProfileSelectParams,
) {
	return db
		.select(columns)
		.from(profilesTable)
		.where(
			and(
				eq(profilesTable.userId, params.userId),
				eq(profilesTable.isDefault, true),
			),
		)
}

export async function insertProfile(db: Database, data: ProfileInsert) {
	return db.insert(profilesTable).values(data).returning(columns)
}

export async function deleteProfile(
	db: Database,
	params: ProfileSelectByIdParams,
) {
	return db
		.delete(profilesTable)
		.where(
			and(
				eq(profilesTable.id, params.id),
				eq(profilesTable.userId, params.userId),
			),
		)
}
