import client, { type ClientOptions } from 'openapi-fetch'
import { AuthAPI } from './auth'
import type { paths } from './openapi'

export class API {
	auth: AuthAPI

	constructor(options: ClientOptions) {
		const c = client<paths>(options)

		this.auth = new AuthAPI(c)
	}
}

export type Client = ReturnType<typeof client<paths>>

export * from './types'
export * from './error'
