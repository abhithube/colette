import type { User, UserCreateData, UsersRepository } from '@colette/core'
import type { Database } from '../client'
import { insertProfile, insertUser } from '../queries'

export class UsersPostgresRepository implements UsersRepository {
	constructor(private db: Database) {}

	async create(data: UserCreateData): Promise<User> {
		const userRows = await insertUser(this.db, data.user)
		if (userRows.length === 0) {
			throw new Error('User not created')
		}
		const user = userRows[0]

		const profileRows = await insertProfile(this.db, {
			id: data.profile.id,
			title: 'Default',
			isDefault: true,
			userId: user.id,
		})
		if (profileRows.length === 0) {
			throw new Error('Profile not created')
		}

		return user
	}
}
