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

export type FindManyFeedsParams = {
	profileId: string
}

export type ParseOptions = {
	feedLinkExpr?: string
	feedTitleExpr: string
	feedEntriesExpr: string
	entryLinkExpr: string
	entryTitleExpr: string
	entryPublishedExpr: string
	entryDescriptionExpr?: string
	entryAuthorExpr?: string
	entryThumbnailExpr?: string
}

export type ParsedFeed = {
	link: string
	title: string
	entries: ParsedEntry[]
}

export type ParsedEntry = {
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
