import type { ParseOptions } from './scraper'

export const RSS_OPTIONS: ParseOptions = {
	linkExpr: '/rss/channel/link/text()',
	titleExpr: '/rss/channel/title/text()',
	entriesExpr: '/rss/channel/item',
	entryLinkExpr: '/link/text()',
	entryTitleExpr: '/title/text()',
	entryPublishedExpr: '/pubDate/text()',
	entryDescriptionExpr: '/description/text()',
	entryAuthorExpr: '/author/text()',
	entryThumbnailExpr: "/enclosure[starts-with(@type, 'image/')]/@url",
}
