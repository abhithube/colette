import type {
  AuthAPI,
  BackupAPI,
  BookmarkAPI,
  FeedAPI,
  FeedEntryAPI,
  ProfileAPI,
  TagAPI,
} from '@colette/core'
import {
  AuthCommands,
  FeedCommands,
  FeedEntryCommands,
  ProfileCommands,
} from './commands'

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
    this.feedEntries = new FeedEntryCommands()
    this.feeds = new FeedCommands()
    this.profiles = new ProfileCommands()
    this.tags = undefined as any
  }
}
