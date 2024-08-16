import createClient, { type ClientOptions } from 'openapi-fetch'
import { AuthAPI } from './auth'
import { BookmarkAPI } from './bookmark'
import { FeedAPI } from './feed'
import { FeedEntryAPI } from './feed-entry'
import type { paths } from './openapi'
import { ProfileAPI } from './profile'
import { TagAPI } from './tag'

export class API {
  auth: AuthAPI
  bookmarks: BookmarkAPI
  entries: FeedEntryAPI
  feeds: FeedAPI
  profiles: ProfileAPI
  tags: TagAPI

  constructor(options: ClientOptions) {
    const client = createClient<paths>(options)

    this.auth = new AuthAPI(client)
    this.bookmarks = new BookmarkAPI(client)
    this.entries = new FeedEntryAPI(client)
    this.feeds = new FeedAPI(client)
    this.profiles = new ProfileAPI(client)
    this.tags = new TagAPI(client)
  }
}

export type Client = ReturnType<typeof createClient<paths>>

export * from './types'
export * from './error'
