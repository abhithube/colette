import createClient, { type ClientOptions } from 'openapi-fetch'
import { AuthAPI } from './auth'
import { BookmarksAPI } from './bookmarks'
import { EntriesAPI } from './entries'
import { FeedsAPI } from './feeds'
import type { paths } from './openapi'
import { ProfilesAPI } from './profiles'
import { TagsAPI } from './tags'

export class API {
	auth: AuthAPI
	bookmarks: BookmarksAPI
	entries: EntriesAPI
	feeds: FeedsAPI
	profiles: ProfilesAPI
	tags: TagsAPI

	constructor(options: ClientOptions) {
		const client = createClient<paths>(options)

		this.auth = new AuthAPI(client)
		this.bookmarks = new BookmarksAPI(client)
		this.entries = new EntriesAPI(client)
		this.feeds = new FeedsAPI(client)
		this.profiles = new ProfilesAPI(client)
		this.tags = new TagsAPI(client)
	}
}

export type Client = ReturnType<typeof createClient<paths>>

export * from './types'
export * from './error'
