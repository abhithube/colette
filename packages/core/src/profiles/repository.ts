import type { Profile } from './model'

export interface ProfilesRepository {
	create(data: ProfileCreateData): Promise<Profile>
}

export type ProfileCreateData = {
	id: string
	title: string
	imageUrl?: string
	isDefault?: boolean
	userId: string
}
