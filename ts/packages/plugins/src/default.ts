import type { ExtractedFeed } from '@colette/core'
import {
	ATOM_OPTIONS,
	FeedScraper,
	RSS_OPTIONS,
	evaluate,
} from '@colette/scraper'

export class DefaultFeedScraper extends FeedScraper {
	constructor() {
		super({
			feedTitleExpr: '',
			feedEntriesExpr: '',
			entryLinkExpr: '',
			entryTitleExpr: '',
		})
	}

	extract(feedUrl: string, document: Document): ExtractedFeed {
		if (evaluate('/rss', document).iterateNext()) {
			this.options = RSS_OPTIONS
		} else if (evaluate('/feed', document).iterateNext()) {
			this.options = ATOM_OPTIONS
		} else {
			throw new Error('Unsupported feed type')
		}

		return super.extract(feedUrl, document)
	}
}
