import type { Client } from '.'
import { APIError } from './error'
import type { components } from './openapi'
import type { Profile } from './types'

export type Login = components['schemas']['Login']

export class AuthAPI {
	constructor(private client: Client) {}

	async login(body: Login): Promise<Profile> {
		const res = await this.client.POST('/api/v1/auth/login', {
			body,
		})
		if (res.error) {
			throw new APIError(res.error.message)
		}

		return res.data
	}
}
