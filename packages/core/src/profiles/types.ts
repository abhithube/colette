export type Profile = {
	id: string
	title: string
	imageUrl: string | null
	createdAt: string
	updatedAt: string
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
