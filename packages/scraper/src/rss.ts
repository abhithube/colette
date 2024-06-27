import type { ParseOptions } from '@colette/core'

export const RSS_OPTIONS: ParseOptions = {
	feedLinkExpr: '/rss/channel/link/text()',
	feedTitleExpr: '/rss/channel/title/text()',
	feedEntriesExpr: '/rss/channel/item',
	entryLinkExpr: '/link/text()',
	entryTitleExpr: '/title/text()',
	entryPublishedExpr: '/pubDate/text()',
	entryDescriptionExpr: '/description/text()',
	entryAuthorExpr: '/author/text()',
	entryThumbnailExpr: "/enclosure[starts-with(@type, 'image/')]/@url",
}
