import type { FetchOptions } from 'openapi-fetch'
import type { Client } from '.'
import { APIError } from './error'
import type { operations } from './openapi'
import type { EntryList, ListEntriesQuery } from './types'

export class EntriesAPI {
	constructor(private client: Client) {}

	async list(
		query?: ListEntriesQuery,
		options?: FetchOptions<operations['listEntries']>,
	): Promise<EntryList> {
		const res = await this.client.GET('/entries', {
			params: {
				query,
			},
			...options,
		})
		if (res.error) {
			throw new APIError('unknown error')
		}

		return res.data
	}
}
