import {
  type ApiClient,
  FeedEntry,
  FeedEntryUpdate,
  Paginated_FeedEntry,
  get_ListFeedEntries,
} from './openapi.gen'
import type { z } from 'zod'

export const FeedEntryListQuery = get_ListFeedEntries.parameters.shape.query
export type FeedEntryListQuery = z.infer<typeof FeedEntryListQuery>

export type FeedEntryList = Paginated_FeedEntry
export const FeedEntryList = Paginated_FeedEntry

export interface FeedEntryAPI {
  list(query: FeedEntryListQuery): Promise<FeedEntryList>

  update(id: string, data: FeedEntryUpdate): Promise<FeedEntry>
}

export class HTTPFeedEntryAPI implements FeedEntryAPI {
  constructor(private client: ApiClient) {}

  list(query: FeedEntryListQuery): Promise<FeedEntryList> {
    return this.client
      .get('/feedEntries', {
        query: FeedEntryListQuery.parse(query),
      })
      .then(FeedEntryList.parse)
  }

  update(id: string, data: FeedEntryUpdate): Promise<FeedEntry> {
    return this.client
      .patch('/feedEntries/{id}', {
        path: {
          id,
        },
        body: FeedEntryUpdate.parse(data),
      })
      .then(FeedEntry.parse)
  }
}
