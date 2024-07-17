import type { Client } from '.'
import { APIError, UnauthorizedError, UnprocessableContentError } from './error'
import type { Login, Profile } from './types'

export class AuthAPI {
	constructor(private client: Client) {}

	async login(body: Login): Promise<Profile> {
		const res = await this.client.POST('/auth/login', {
			body,
		})
		if (res.error) {
			if (res.response.status === 401) {
				throw new UnauthorizedError(res.error.message)
			}

			if (res.response.status === 422) {
				throw new UnprocessableContentError(res.error.message)
			}

			throw new APIError(res.error.message)
		}

		return res.data
	}
}
