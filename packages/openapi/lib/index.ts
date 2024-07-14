import createClient, { type ClientOptions } from 'openapi-fetch'
import { AuthAPI } from './auth'
import { FeedsAPI } from './feeds'
import type { paths } from './openapi'

export class API {
	auth: AuthAPI
	feeds: FeedsAPI

	constructor(options: ClientOptions) {
		const client = createClient<paths>(options)

		this.auth = new AuthAPI(client)
		this.feeds = new FeedsAPI(client)
	}
}

export type Client = ReturnType<typeof createClient<paths>>

export * from './types'
export * from './error'
