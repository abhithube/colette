import type { ParseOptions } from './scraper'

export const ATOM_OPTIONS: ParseOptions = {
	linkExpr: "/feed/link[@rel='alternate']/@href",
	titleExpr: '/feed/title/text()',
	entriesExpr: '/feed/entry',
	entryLinkExpr: './link/@href',
	entryTitleExpr: './title/text()',
	entryPublishedExpr: './published/text()',
	entryDescriptionExpr: './content/text()',
	entryAuthorExpr: './author/name/text()',
}
