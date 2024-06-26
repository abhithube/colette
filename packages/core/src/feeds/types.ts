export type Feed = {
	id: string
	link: string
	title: string
	url: string | null
	customTitle: string | null
	createdAt: Date
	updatedAt: Date
	unreadCount?: number
}

export type FindManyFeedsParams = {
	profileId: string
}
