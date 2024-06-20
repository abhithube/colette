export type ScraperOptions = {
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
	scrape(options: ScraperOptions, document?: Document): T
}
