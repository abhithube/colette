import type {
	Profile,
	ProfileCreateData,
	ProfilesRepository,
} from '@colette/core'
import type { Database } from '../client'
import { insertProfile } from '../queries'

export class ProfilesPostgresRepository implements ProfilesRepository {
	constructor(private db: Database) {}

	async create(data: ProfileCreateData): Promise<Profile> {
		const rows = await insertProfile(this.db, data)
		if (rows.length === 0) {
			throw new Error('Profile not created')
		}

		return rows[0]
	}
}
