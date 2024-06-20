import type { ParseOptions, Scraper } from './scraper'
import { evaluate, evaluateString } from './utils'

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
	link: string
	title: string
	entries: ProcessedEntry[]
}

export type ProcessedEntry = {
	link: string
	title: string
	published?: Date
	description?: string
	author?: string
	thumbnail?: string
}

export class FeedScraper implements Scraper<ParsedFeed, ProcessedFeed> {
	constructor(private options: ParseOptions) {}

	parse(feedUrl: string, document: Document) {
		let link = feedUrl
		if (this.options.linkExpr) {
			link = evaluateString(this.options.linkExpr, document)
		}
		const title = evaluateString(this.options.titleExpr, document)
		const entryNodes = evaluate(this.options.entriesExpr, document)

		const entries: ParsedEntry[] = []

		let node = entryNodes.iterateNext()
		while (node) {
			const link = evaluateString(this.options.entryLinkExpr, document, node)
			const title = evaluateString(this.options.entryTitleExpr, document, node)
			const published = evaluateString(
				this.options.entryPublishedExpr,
				document,
				node,
			)
			const description =
				this.options.entryDescriptionExpr &&
				evaluateString(this.options.entryDescriptionExpr, document, node)
			const author =
				this.options.entryAuthorExpr &&
				evaluateString(this.options.entryAuthorExpr, document, node)
			const thumbnail =
				this.options.entryThumbnailExpr &&
				evaluateString(this.options.entryThumbnailExpr, document, node)

			entries.push({
				link,
				title,
				published,
				description,
				author,
				thumbnail,
			})

			node = entryNodes.iterateNext()
		}

		return {
			link,
			title,
			entries,
		}
	}

	postprocess(feedUrl: string, parsed: ParsedFeed): ProcessedFeed {
		return {
			link: new URL(parsed.link).href,
			title: parsed.title,
			entries: parsed.entries.map((parsed) => {
				return {
					link: new URL(parsed.link).href,
					title: parsed.title,
					published: parsed.published ? new Date(parsed.published) : undefined,
					description: parsed.description,
					author: parsed.author,
					thumbnail: parsed.thumbnail
						? new URL(parsed.thumbnail).href
						: undefined,
				}
			}),
		}
	}
}
