import type {
	FindManyProfilesParams,
	FindOneProfileOrDefaultParams,
	FindOneProfileParams,
	Profile,
	ProfileCreateData,
} from './types'

export interface ProfilesRepository {
	findMany(params: FindManyProfilesParams): Promise<Profile[]>

	findOne(params: FindOneProfileOrDefaultParams): Promise<Profile | null>

	create(data: ProfileCreateData): Promise<Profile>

	delete(params: FindOneProfileParams): Promise<boolean>
}
