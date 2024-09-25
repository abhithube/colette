import type {
  AuthAPI,
  BackupAPI,
  BookmarkAPI,
  FeedAPI,
  FeedEntryAPI,
  ProfileAPI,
  TagAPI,
} from '@colette/core'
import { AuthCommands, ProfileCommands } from './commands'

export interface API {
  auth: AuthAPI
  backups: BackupAPI
  bookmarks: BookmarkAPI
  feedEntries: FeedEntryAPI
  feeds: FeedAPI
  profiles: ProfileAPI
  tags: TagAPI
}

export class CommandsAPI implements API {
  auth: AuthAPI
  backups: BackupAPI
  bookmarks: BookmarkAPI
  feedEntries: FeedEntryAPI
  feeds: FeedAPI
  profiles: ProfileAPI
  tags: TagAPI

  constructor() {
    this.auth = new AuthCommands()
    this.backups = undefined as any
    this.bookmarks = undefined as any
    this.feedEntries = undefined as any
    this.feeds = undefined as any
    this.profiles = new ProfileCommands()
    this.tags = undefined as any
  }
}
