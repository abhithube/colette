import type { User, UserCreateData, UsersRepository } from '@colette/core'
import type { Database } from '../client'
import { insertProfile, insertUser } from '../queries'

export class UsersPostgresRepository implements UsersRepository {
	constructor(private db: Database) {}

	async create(data: UserCreateData): Promise<User> {
		return this.db.transaction(async (tx) => {
			const [user] = await insertUser(tx, data.user)
			if (!user) {
				throw new Error('User not created')
			}

			const [profile] = await insertProfile(tx, {
				id: data.profile.id,
				title: 'Default',
				isDefault: true,
				userId: user.id,
			})
			if (!profile) {
				throw new Error('Profile not created')
			}

			return user
		})
	}
}
