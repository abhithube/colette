import type { Client } from '.'
import { BaseError } from './error'
import type { Login, Profile } from './types'

export class AuthAPI {
	constructor(private client: Client) {}

	async login(body: Login): Promise<Profile> {
		const res = await this.client.POST('/api/v1/auth/login', {
			body,
		})
		if (res.error) {
			throw new BaseError(res.error.message)
		}

		return res.data
	}
}
