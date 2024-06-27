import type { HttpClient } from '@colette/core'

export class FetchClient implements HttpClient {
	async get(url: string | URL | Request): Promise<Response> {
		return fetch(url)
	}
}
