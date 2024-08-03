import type { FetchOptions } from 'openapi-fetch'
import type { Client } from '.'
import {
	APIError,
	BadGatewayError,
	NotFoundError,
	UnprocessableContentError,
} from './error'
import type { operations } from './openapi'
import type {
	Bookmark,
	BookmarkCreate,
	BookmarkList,
	BookmarkUpdate,
	ListBookmarksQuery,
} from './types'

export class BookmarksAPI {
	constructor(private client: Client) {}

	async list(
		query?: ListBookmarksQuery,
		options?: Omit<FetchOptions<operations['listBookmarks']>, 'params'>,
	): Promise<BookmarkList> {
		const res = await this.client.GET('/bookmarks', {
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

	async get(
		id: string,
		options?: Omit<FetchOptions<operations['getBookmark']>, 'params'>,
	): Promise<Bookmark> {
		const res = await this.client.GET('/bookmarks/{id}', {
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

			throw new APIError(res.error.message)
		}

		return res.data
	}

	async create(
		body: BookmarkCreate,
		options?: Omit<FetchOptions<operations['createBookmark']>, 'body'>,
	): Promise<Bookmark> {
		const res = await this.client.POST('/bookmarks', {
			body,
			...options,
		})
		if (res.error) {
			if (res.response.status === 422) {
				throw new UnprocessableContentError(res.error.message)
			}
			if (res.response.status === 502) {
				throw new BadGatewayError(res.error.message)
			}

			throw new APIError(res.error.message)
		}

		return res.data
	}

	async update(
		id: string,
		body: BookmarkUpdate,
		options?: Omit<
			FetchOptions<operations['updateBookmark']>,
			'params' | 'body'
		>,
	): Promise<Bookmark> {
		const res = await this.client.PATCH('/bookmarks/{id}', {
			params: {
				path: {
					id,
				},
			},
			body,
			...options,
		})
		if (res.error) {
			if (res.response.status === 404) {
				throw new NotFoundError(res.error.message)
			}
			if (res.response.status === 422) {
				throw new UnprocessableContentError(res.error.message)
			}

			throw new APIError(res.error.message)
		}

		return res.data
	}

	async delete(
		id: string,
		options?: Omit<FetchOptions<operations['deleteBookmark']>, 'params'>,
	): Promise<void> {
		const res = await this.client.DELETE('/bookmarks/{id}', {
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

			throw new APIError(res.error.message)
		}
	}
}
