import type {
  AuthAPI,
  BackupAPI,
  BookmarkAPI,
  CollectionAPI,
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
import { CollectionCommands } from './commands/collection'

export interface API {
  auth: AuthAPI
  backups: BackupAPI
  bookmarks: BookmarkAPI
  collections: CollectionAPI
  feedEntries: FeedEntryAPI
  feeds: FeedAPI
  smartFeeds: SmartFeedAPI
  tags: TagAPI
}

export class CommandsAPI implements API {
  auth: AuthAPI
  backups: BackupAPI
  bookmarks: BookmarkAPI
  collections: CollectionAPI
  feedEntries: FeedEntryAPI
  feeds: FeedAPI
  smartFeeds: SmartFeedAPI
  tags: TagAPI

  constructor() {
    this.auth = new AuthCommands()
    this.backups = new BackupCommands()
    this.bookmarks = new BookmarkCommands()
    this.collections = new CollectionCommands()
    this.feedEntries = new FeedEntryCommands()
    this.feeds = new FeedCommands()
    this.smartFeeds = new SmartFeedCommands()
    this.tags = new TagCommands()
  }
}
