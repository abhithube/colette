import type { Scraper, ScraperOptions } from './scraper'
import { evaluate, evaluateString } from './utils'

export type Feed = {
	link: string
	title: string
	entries: Entry[]
}

export type Entry = {
	link: string
	title: string
	published?: string
	description?: string
	author?: string
	thumbnail?: string
}

export class FeedScraper implements Scraper<Feed> {
	scrape(options: ScraperOptions, document: Document) {
		const link = evaluateString(options.linkExpr, document)
		const title = evaluateString(options.titleExpr, document)
		const entryNodes = evaluate(options.entriesExpr, document)

		const entries: Entry[] = []

		let node = entryNodes.iterateNext()
		while (node) {
			const link = evaluateString(options.entryLinkExpr, document, node)
			const title = evaluateString(options.entryTitleExpr, document, node)
			const published = evaluateString(
				options.entryPublishedExpr,
				document,
				node,
			)
			const description =
				options.entryDescriptionExpr &&
				evaluateString(options.entryDescriptionExpr, document, node)
			const author =
				options.entryAuthorExpr &&
				evaluateString(options.entryAuthorExpr, document, node)
			const thumbnail =
				options.entryThumbnailExpr &&
				evaluateString(options.entryThumbnailExpr, document, node)

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
}
