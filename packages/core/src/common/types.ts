export type Paginated<T> = {
	hasMore: boolean
	data: T[]
}

export type FindOneParams = {
	id: string
	profileId: string
}
