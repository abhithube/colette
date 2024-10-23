import {
  FeedEntry,
  type FeedEntryAPI,
  FeedEntryList,
  FeedEntryListQuery,
  FeedEntryUpdate,
} from '@colette/core'
import { invoke } from '@tauri-apps/api/core'

export class FeedEntryCommands implements FeedEntryAPI {
  list(query: FeedEntryListQuery): Promise<FeedEntryList> {
    return invoke('list_feed_entries', {
      query: FeedEntryListQuery.parse(query),
    }).then(FeedEntryList.parse)
  }

  get(id: string): Promise<FeedEntry> {
    return invoke('get_feed_entry', { id }).then(FeedEntry.parse)
  }

  update(id: string, data: FeedEntryUpdate): Promise<FeedEntry> {
    return invoke('update_feed_entry', {
      id,
      data: FeedEntryUpdate.parse(data),
    }).then(FeedEntry.parse)
  }
}
