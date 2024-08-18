import type { z } from 'zod'
import {
  type ApiClient,
  FeedEntry,
  FeedEntryList,
  FeedEntryUpdate,
  get_ListFeedEntries,
} from './openapi.gen'

const ListFeedEntriesQuery = get_ListFeedEntries.parameters.shape.query
export type ListFeedEntriesQuery = z.infer<typeof ListFeedEntriesQuery>

export interface FeedEntryAPI {
  list(query: ListFeedEntriesQuery): Promise<FeedEntryList>

  update(id: string, data: FeedEntryUpdate): Promise<FeedEntry>
}

export class HTTPFeedEntryAPI implements FeedEntryAPI {
  constructor(private client: ApiClient) {}

  async list(query: ListFeedEntriesQuery): Promise<FeedEntryList> {
    return this.client
      .get('/feedEntries', {
        query: await ListFeedEntriesQuery.parseAsync(query),
      })
      .then(FeedEntryList.parseAsync)
  }

  async update(id: string, data: FeedEntryUpdate): Promise<FeedEntry> {
    return this.client
      .patch('/feedEntries/{id}', {
        path: {
          id,
        },
        body: await FeedEntryUpdate.parseAsync(data),
      })
      .then(FeedEntry.parseAsync)
  }
}
