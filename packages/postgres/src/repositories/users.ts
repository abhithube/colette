import type { CreateData, User, UsersRepository } from '@colette/core'
import type { Database } from '../client'
import { insertUser } from '../queries'

export class UsersPostgresRepository implements UsersRepository {
	constructor(private db: Database) {}

	async create(data: CreateData): Promise<User> {
		const rows = await insertUser(this.db, data)
		if (rows.length === 0) {
			throw new Error('User not created')
		}

		return rows[0]
	}
}
