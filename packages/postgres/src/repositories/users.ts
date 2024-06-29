import type {
	FindOneUserParams,
	User,
	UserCreateData,
	UsersRepository,
} from '@colette/core'
import { nanoid } from 'nanoid'
import type { Database } from '../client'
import { insertProfile, insertUser, selectUserByEmail } from '../queries'

export class UsersPostgresRepository implements UsersRepository {
	constructor(private readonly db: Database) {}

	async findOne(params: FindOneUserParams): Promise<User | null> {
		const [user] = await selectUserByEmail(this.db, params)
		if (!user) {
			return null
		}

		return user
	}

	async create(data: UserCreateData): Promise<User> {
		return this.db.transaction(async (tx) => {
			const [user] = await insertUser(tx, {
				...data,
				id: nanoid(),
			})
			if (!user) {
				throw new Error('User not created')
			}

			const [profile] = await insertProfile(tx, {
				id: nanoid(),
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
