export type ParseOptions = {
	linkExpr?: string
	titleExpr: string
	entriesExpr: string
	entryLinkExpr: string
	entryTitleExpr: string
	entryPublishedExpr: string
	entryDescriptionExpr?: string
	entryAuthorExpr?: string
	entryThumbnailExpr?: string
}

export interface Scraper<T, U> {
	prepare(feedUrl: string): Request
	parse(feedUrl: string, document: Document): T
	postprocess(feedUrl: string, parsed: T): U
}
