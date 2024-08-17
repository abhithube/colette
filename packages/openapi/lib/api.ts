import type {
  API,
  AuthAPI,
  BookmarkAPI,
  FeedAPI,
  FeedEntryAPI,
  ProfileAPI,
  TagAPI,
} from '@colette/core'
import createClient, { type ClientOptions } from 'openapi-fetch'
import { HTTPAuthAPI } from './auth'
import { HTTPBookmarkAPI } from './bookmark'
import { HTTPFeedAPI } from './feed'
import { HTTPFeedEntryAPI } from './feed-entry'
import type { paths } from './openapi'
import { HTTPProfileAPI } from './profile'
import { HTTPTagAPI } from './tag'

export class HttpAPI implements API {
  auth: AuthAPI
  bookmarks: BookmarkAPI
  feedEntries: FeedEntryAPI
  feeds: FeedAPI
  profiles: ProfileAPI
  tags: TagAPI

  constructor(options: ClientOptions) {
    const client = createClient<paths>(options)

    this.auth = new HTTPAuthAPI(client)
    this.bookmarks = new HTTPBookmarkAPI(client)
    this.feedEntries = new HTTPFeedEntryAPI(client)
    this.feeds = new HTTPFeedAPI(client)
    this.profiles = new HTTPProfileAPI(client)
    this.tags = new HTTPTagAPI(client)
  }
}
