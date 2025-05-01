import { components, operations, paths } from './openapi'
import { Client } from 'openapi-fetch'

export type FeedEntry = components['schemas']['FeedEntry']
export type FeedEntryList = components['schemas']['Paginated_FeedEntry']

export type FeedEntryListQuery = NonNullable<
  operations['listFeedEntries']['parameters']['query']
>

export interface FeedEntryAPI {
  listFeedEntries(query: FeedEntryListQuery): Promise<FeedEntryList>

  getFeedEntry(id: string): Promise<FeedEntry>
}

export class HTTPFeedEntryAPI implements FeedEntryAPI {
  constructor(private client: Client<paths>) {}

  async listFeedEntries(query: FeedEntryListQuery): Promise<FeedEntryList> {
    const res = await this.client.GET('/feedEntries', {
      query,
    })

    return res.data!
  }

  async getFeedEntry(id: string): Promise<FeedEntry> {
    const res = await this.client.GET('/feedEntries/{id}', {
      params: {
        path: {
          id,
        },
      },
    })

    return res.data!
  }
}
