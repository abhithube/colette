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
  BackupCommands,
  BookmarkCommands,
  FeedCommands,
  FeedEntryCommands,
  ProfileCommands,
  TagCommands,
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
    this.backups = new BackupCommands()
    this.bookmarks = new BookmarkCommands()
    this.feedEntries = new FeedEntryCommands()
    this.feeds = new FeedCommands()
    this.profiles = new ProfileCommands()
    this.tags = new TagCommands()
  }
}
