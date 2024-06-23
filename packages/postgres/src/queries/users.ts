import { eq } from 'drizzle-orm'
import type { Database } from '../client'
import { usersTable } from '../schema'
import type { UserInsert, UserSelectByEmailParams } from '../types'

const columns = {
	id: usersTable.id,
	email: usersTable.email,
	password: usersTable.password,
	createdAt: usersTable.createdAt,
	updatedAt: usersTable.updatedAt,
}

export async function selectUserByEmail(
	db: Database,
	params: UserSelectByEmailParams,
) {
	return db
		.select(columns)
		.from(usersTable)
		.where(eq(usersTable.email, params.email))
}

export async function insertUser(db: Database, data: UserInsert) {
	return db.insert(usersTable).values(data).returning(columns)
}
