import {
  type ApiClient,
  FeedEntry,
  Paginated_FeedEntry,
  get_ListFeedEntries,
} from './openapi.gen'
import type { z } from 'zod'

export const FeedEntryListQuery = get_ListFeedEntries.parameters.shape.query
export type FeedEntryListQuery = z.infer<typeof FeedEntryListQuery>

export type FeedEntryList = Paginated_FeedEntry
export const FeedEntryList = Paginated_FeedEntry

export interface FeedEntryAPI {
  listFeedEntries(query: FeedEntryListQuery): Promise<FeedEntryList>

  getFeedEntry(id: string): Promise<FeedEntry>
}

export class HTTPFeedEntryAPI implements FeedEntryAPI {
  constructor(private client: ApiClient) {}

  listFeedEntries(query: FeedEntryListQuery): Promise<FeedEntryList> {
    return this.client
      .get('/feedEntries', {
        query: FeedEntryListQuery.parse(query),
      })
      .then(FeedEntryList.parse)
  }

  getFeedEntry(id: string): Promise<FeedEntry> {
    return this.client
      .get('/feedEntries/{id}', {
        path: {
          id,
        },
      })
      .then(FeedEntry.parse)
  }
}
