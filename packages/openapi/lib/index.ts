import createClient, { type ClientOptions } from 'openapi-fetch'
import { AuthAPI } from './auth'
import { EntriesAPI } from './entries'
import { FeedsAPI } from './feeds'
import type { paths } from './openapi'
import { ProfilesAPI } from './profiles'

export class API {
	auth: AuthAPI
	entries: EntriesAPI
	feeds: FeedsAPI
	profiles: ProfilesAPI

	constructor(options: ClientOptions) {
		const client = createClient<paths>(options)

		this.auth = new AuthAPI(client)
		this.entries = new EntriesAPI(client)
		this.feeds = new FeedsAPI(client)
		this.profiles = new ProfilesAPI(client)
	}
}

export type Client = ReturnType<typeof createClient<paths>>

export * from './types'
export * from './error'
