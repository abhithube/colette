export interface Scraper<T, U> {
	prepare(feedUrl: string): Request

	extract(feedUrl: string, document: Document): T

	postprocess(feedUrl: string, parsed: T): U
}
