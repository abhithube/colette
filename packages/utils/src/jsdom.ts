import type { ResponseParser } from '@colette/core'
import { JSDOM } from 'jsdom'

export class JSDOMParser implements ResponseParser<Document> {
	async parse(res: Response): Promise<Document> {
		const raw = await res.text()
		const contentType = res.headers.get('Content-Type')?.split(';').at(0)

		let dom: JSDOM
		if (
			(contentType &&
				(contentType === 'text/html' ||
					contentType === 'application/xhtml+xml')) ||
			raw.includes('<html')
		) {
			dom = new JSDOM(raw)
		} else if (
			(contentType &&
				(contentType === 'text/xml' ||
					contentType === 'application/xml' ||
					contentType === 'application/rss+xml' ||
					contentType === 'application/atom+xml')) ||
			raw.includes('<rss') ||
			raw.includes('<feed')
		)
			dom = new JSDOM(raw, {
				contentType: 'text/xml',
			})
		else throw new Error('Unsupported document')

		return dom.window.document
	}
}
