import type { ParseOptions } from '@colette/core'

export const ATOM_OPTIONS: ParseOptions = {
	feedLinkExpr: "/feed/link[@rel='alternate']/@href",
	feedTitleExpr: '/feed/title/text()',
	feedEntriesExpr: '/feed/entry',
	entryLinkExpr: './link/@href',
	entryTitleExpr: './title/text()',
	entryPublishedExpr: './published/text()',
	entryDescriptionExpr: './content/text()',
	entryAuthorExpr: './author/name/text()',
}
