export type Entry = {
	id: string
	link: string
	title: string
	publishedAt: string | null
	description: string | null
	author: string | null
	thumbnailUrl: string | null
	hasRead: boolean
	feedId: string
}

export type ListParams = {
	publishedAt?: string
	feedId?: string
}

export type FindManyEntriesParams = {
	profileId: string
	limit: number
	publishedAt?: string
	profileFeedId?: string
}