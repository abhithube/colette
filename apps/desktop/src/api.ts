import type {
  AuthAPI,
  BackupAPI,
  BookmarkAPI,
  FeedAPI,
  FeedEntryAPI,
  SmartFeedAPI,
  TagAPI,
} from '@colette/core'
import {
  AuthCommands,
  BackupCommands,
  BookmarkCommands,
  FeedCommands,
  FeedEntryCommands,
  SmartFeedCommands,
  TagCommands,
} from './commands'

export interface API {
  auth: AuthAPI
  backups: BackupAPI
  bookmarks: BookmarkAPI
  feedEntries: FeedEntryAPI
  feeds: FeedAPI
  smartFeeds: SmartFeedAPI
  tags: TagAPI
}

export class CommandsAPI implements API {
  auth: AuthAPI
  backups: BackupAPI
  bookmarks: BookmarkAPI
  feedEntries: FeedEntryAPI
  feeds: FeedAPI
  smartFeeds: SmartFeedAPI
  tags: TagAPI

  constructor() {
    this.auth = new AuthCommands()
    this.backups = new BackupCommands()
    this.bookmarks = new BookmarkCommands()
    this.feedEntries = new FeedEntryCommands()
    this.feeds = new FeedCommands()
    this.smartFeeds = new SmartFeedCommands()
    this.tags = new TagCommands()
  }
}
