import type { z } from 'zod'
import {
  type ApiClient,
  FeedEntry,
  FeedEntryList,
  FeedEntryUpdate,
  get_ListFeedEntries,
} from './openapi.gen'

export const FeedEntryListQuery = get_ListFeedEntries.parameters.shape.query
export type FeedEntryListQuery = z.infer<typeof FeedEntryListQuery>

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
