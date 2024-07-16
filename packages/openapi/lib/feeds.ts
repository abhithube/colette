import type { FetchOptions } from 'openapi-fetch'
import type { Client } from '.'
import {
	BadGatewayError,
	BaseError,
	NotFoundError,
	UnprocessableContentError,
} from './error'
import type { operations } from './openapi'
import type { CreateFeed, Feed, FeedList } from './types'

export class FeedsAPI {
	constructor(private client: Client) {}

	async list(
		options?: FetchOptions<operations['listFeeds']>,
	): Promise<FeedList> {
		const res = await this.client.GET('/feeds', options)
		if (res.error) {
			throw new BaseError('unknown error')
		}

		return res.data
	}

	async get(
		id: string,
		options?: Omit<FetchOptions<operations['getFeed']>, 'params'>,
	): Promise<Feed> {
		const res = await this.client.GET('/feeds/{id}', {
			params: {
				path: {
					id,
				},
			},
			...options,
		})
		if (res.error) {
			if (res.response.status === 404) {
				throw new NotFoundError(res.error.message)
			}

			throw new BaseError(res.error.message)
		}

		return res.data
	}

	async create(
		body: CreateFeed,
		options?: Omit<FetchOptions<operations['createFeed']>, 'body'>,
	): Promise<Feed> {
		const res = await this.client.POST('/feeds', {
			body,
			...options,
		})
		if (res.error) {
			if (res.response.status === 422) {
				throw new UnprocessableContentError(res.error as any)
			}
			if (res.response.status === 502) {
				throw new BadGatewayError(res.error.message)
			}

			throw new BaseError(res.error.message)
		}

		return res.data
	}

	async delete(
		id: string,
		options?: Omit<FetchOptions<operations['deleteFeed']>, 'params'>,
	): Promise<void> {
		const res = await this.client.DELETE('/feeds/{id}', {
			params: {
				path: {
					id,
				},
			},
			...options,
		})
		if (res.error) {
			if (res.response.status === 404) {
				throw new NotFoundError(res.error.message)
			}

			throw new BaseError(res.error.message)
		}
	}
}
