import type { Database } from '../client'
import { profilesTable } from '../schema'
import type { ProfileInsert } from '../types'

const columns = {
	id: profilesTable.id,
	title: profilesTable.title,
	imageUrl: profilesTable.imageUrl,
	createdAt: profilesTable.createdAt,
	updatedAt: profilesTable.updatedAt,
}

export async function insertProfile(db: Database, data: ProfileInsert) {
	return db.insert(profilesTable).values(data).returning(columns)
}
