import type { Profile } from './model'

export interface ProfilesRepository {
	findMany(params: FindManyProfilesParams): Promise<Profile[]>

	findOne(params: FindOneProfileOrDefaultParams): Promise<Profile | null>

	create(data: ProfileCreateData): Promise<Profile>

	delete(params: FindOneProfileParams): Promise<boolean>
}

export type FindManyProfilesParams = {
	userId: string
}

export type FindOneProfileParams = {
	id: string
	userId: string
}

export type FindOneProfileOrDefaultParams = Omit<FindOneProfileParams, 'id'> & {
	id?: string
}

export type ProfileCreateData = {
	id: string
	title: string
	imageUrl?: string
	isDefault?: boolean
	userId: string
}
