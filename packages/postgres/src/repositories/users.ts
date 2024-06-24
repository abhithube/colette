import type { User, UserCreateData, UsersRepository } from '@colette/core'
import type { Database } from '../client'
import { insertUser } from '../queries'

export class UsersPostgresRepository implements UsersRepository {
	constructor(private db: Database) {}

	async create(data: UserCreateData): Promise<User> {
		const rows = await insertUser(this.db, data)
		if (rows.length === 0) {
			throw new Error('User not created')
		}

		return rows[0]
	}
}
