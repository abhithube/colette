export type ParseOptions = {
	linkExpr: string
	titleExpr: string
	entriesExpr: string
	entryLinkExpr: string
	entryTitleExpr: string
	entryPublishedExpr: string
	entryDescriptionExpr?: string
	entryAuthorExpr?: string
	entryThumbnailExpr?: string
}

export interface Scraper<T> {
	parse(options: ParseOptions, document: Document): T
}
