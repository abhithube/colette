export type Feed = {
	id: string
	link: string
	title: string
	url: string | null
	customTitle: string | null
	createdAt: string
	updatedAt: string
	unreadCount?: number
}

export type FeedDto = {
	url: string
}

export type FindManyFeedsParams = {
	profileId: string
}

export type FeedCreateData = {
	feedUrl: string
	processed: ProcessedFeed
	profileId: string
}

export type ParseOptions = {
	feedLinkExpr?: string
	feedTitleExpr: string
	feedEntriesExpr: string
	entryLinkExpr: string
	entryTitleExpr: string
	entryPublishedExpr?: string
	entryDescriptionExpr?: string
	entryAuthorExpr?: string
	entryThumbnailExpr?: string
}

export type ExtractedFeed = {
	link: string
	title: string
	entries: ExtractedEntry[]
}

export type ExtractedEntry = {
	link: string
	title: string
	published?: string
	description?: string
	author?: string
	thumbnail?: string
}

export type ProcessedFeed = {
	link: URL
	title: string
	entries: ProcessedEntry[]
}

export type ProcessedEntry = {
	link: URL
	title: string
	published?: Date
	description?: string
	author?: string
	thumbnail?: URL
}
