import type { Client } from '.'
import {
	APIError,
	ConflictError,
	UnauthorizedError,
	UnprocessableContentError,
} from './error'
import type { Login, Profile, Register, User } from './types'

export class AuthAPI {
	constructor(private client: Client) {}

	async register(body: Register): Promise<User> {
		const res = await this.client.POST('/auth/register', {
			body,
		})
		if (res.error) {
			if (res.response.status === 401) {
				throw new UnauthorizedError(res.error.message)
			}
			if (res.response.status === 409) {
				throw new ConflictError(res.error.message)
			}

			throw new APIError(res.error.message)
		}

		return res.data
	}

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

	async getActive(): Promise<User> {
		const res = await this.client.GET('/auth/@me')
		if (res.error) {
			throw new APIError('unknown error')
		}

		return res.data
	}
}
