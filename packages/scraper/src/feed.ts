import type {
	ExtractedEntry,
	ExtractedFeed,
	ParseOptions,
	ProcessedFeed,
	Scraper,
} from '@colette/core'
import { evaluate, evaluateString } from './utils'

export class FeedScraper implements Scraper<ExtractedFeed, ProcessedFeed> {
	constructor(private readonly options: ParseOptions) {}

	prepare(feedUrl: string): Request {
		return new Request(feedUrl)
	}

	extract(feedUrl: string, document: Document): ExtractedFeed {
		let link = feedUrl
		if (this.options.feedLinkExpr) {
			link = evaluateString(this.options.feedLinkExpr, document)
		}
		const title = evaluateString(this.options.feedTitleExpr, document)
		const entryNodes = evaluate(this.options.feedEntriesExpr, document)

		const entries: ExtractedEntry[] = []

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

	postprocess(feedUrl: string, parsed: ExtractedFeed): ProcessedFeed {
		return {
			link: new URL(parsed.link),
			title: parsed.title,
			entries: parsed.entries.map((parsed) => {
				return {
					link: new URL(parsed.link),
					title: parsed.title,
					published: parsed.published ? new Date(parsed.published) : undefined,
					description: parsed.description,
					author: parsed.author,
					thumbnail: parsed.thumbnail ? new URL(parsed.thumbnail) : undefined,
				}
			}),
		}
	}
}
