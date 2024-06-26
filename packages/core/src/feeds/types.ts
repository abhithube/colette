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
