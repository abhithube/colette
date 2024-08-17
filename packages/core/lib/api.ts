import type { AuthAPI } from './auth'
import type { BookmarkAPI } from './bookmark'
import type { FeedAPI } from './feed'
import type { FeedEntryAPI } from './feed-entry'
import type { ProfileAPI } from './profile'
import type { TagAPI } from './tag'

export interface API {
  auth: AuthAPI
  bookmarks: BookmarkAPI
  feedEntries: FeedEntryAPI
  feeds: FeedAPI
  profiles: ProfileAPI
  tags: TagAPI
}
