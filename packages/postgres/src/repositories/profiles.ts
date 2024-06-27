import type {
	FindManyProfilesParams,
	FindOneProfileOrDefaultParams,
	FindOneProfileParams,
	Profile,
	ProfileCreateData,
	ProfilesRepository,
	ValueGenerator,
} from '@colette/core'
import type { Database } from '../client'
import {
	deleteProfile,
	insertProfile,
	selectDefaultProfile,
	selectProfileById,
	selectProfiles,
} from '../queries'

export class ProfilesPostgresRepository implements ProfilesRepository {
	constructor(
		private readonly db: Database,
		private readonly idGenerator: ValueGenerator<string>,
	) {}

	async findMany(params: FindManyProfilesParams): Promise<Profile[]> {
		return selectProfiles(this.db, params)
	}

	async findOne(
		params: FindOneProfileOrDefaultParams,
	): Promise<Profile | null> {
		const [profile] = params.id
			? await selectProfileById(this.db, {
					id: params.id,
					userId: params.userId,
				})
			: await selectDefaultProfile(this.db, params)
		if (!profile) {
			return null
		}

		return profile
	}

	async create(data: ProfileCreateData): Promise<Profile> {
		const [profile] = await insertProfile(this.db, {
			...data,
			id: this.idGenerator.generate(),
		})
		if (!profile) {
			throw new Error('Profile not created')
		}

		return profile
	}

	async delete(params: FindOneProfileParams): Promise<boolean> {
		const result = await deleteProfile(this.db, params)

		return result.rowCount === 1
	}
}
