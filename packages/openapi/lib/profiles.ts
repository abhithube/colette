import type { FetchOptions } from 'openapi-fetch'
import type { Client } from '.'
import {
	APIError,
	ConflictError,
	NotFoundError,
	UnprocessableContentError,
} from './error'
import type { operations } from './openapi'
import type { Profile, ProfileCreate, ProfileList } from './types'

export class ProfilesAPI {
	constructor(private client: Client) {}

	async list(
		options?: FetchOptions<operations['listProfiles']>,
	): Promise<ProfileList> {
		const res = await this.client.GET('/profiles', options)
		if (res.error) {
			throw new APIError('unknown error')
		}

		return res.data
	}

	async getActive(
		options?: Omit<FetchOptions<operations['getActiveProfile']>, 'params'>,
	): Promise<Profile> {
		const res = await this.client.GET('/profiles/@me', options)
		if (res.error) {
			throw new APIError('unknown error')
		}

		return res.data
	}

	async create(
		body: ProfileCreate,
		options?: Omit<FetchOptions<operations['createProfile']>, 'body'>,
	): Promise<Profile> {
		const res = await this.client.POST('/profiles', {
			body,
			...options,
		})
		if (res.error) {
			if (res.response.status === 422) {
				throw new UnprocessableContentError(res.error.message)
			}

			throw new APIError('unknown error')
		}

		return res.data
	}

	async delete(
		id: string,
		options?: Omit<FetchOptions<operations['deleteProfile']>, 'params'>,
	): Promise<void> {
		const res = await this.client.DELETE('/profiles/{id}', {
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
			if (res.response.status === 409) {
				throw new ConflictError(res.error.message)
			}

			throw new APIError(res.error.message)
		}
	}
}
