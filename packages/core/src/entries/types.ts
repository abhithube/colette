export type Entry = {
	id: string
	link: string
	title: string
	publishedAt: Date | null
	description: string | null
	author: string | null
	thumbnailUrl: string | null
	hasRead: boolean
	feedId: string
}

export type FindManyEntriesParams = {
	profileId: string
	publishedAt?: Date
	profileFeedId?: string
}
