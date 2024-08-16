import type { FetchOptions } from 'openapi-fetch'
import type { Client } from '.'
import { APIError, NotFoundError } from './error'
import type { operations } from './openapi'
import type {
  FeedEntry,
  FeedEntryList,
  FeedEntryUpdate,
  ListFeedEntriesQuery,
} from './types'

export class FeedEntryAPI {
  constructor(private client: Client) {}

  async list(
    query?: ListFeedEntriesQuery,
    options?: Omit<FetchOptions<operations['listFeedEntries']>, 'params'>,
  ): Promise<FeedEntryList> {
    const res = await this.client.GET('/feedEntries', {
      params: {
        query,
      },
      ...options,
    })
    if (res.error) {
      throw new APIError('unknown error')
    }

    return res.data
  }

  async update(
    id: string,
    body: FeedEntryUpdate,
    options?: Omit<
      FetchOptions<operations['updateFeedEntry']>,
      'params' | 'body'
    >,
  ): Promise<FeedEntry> {
    const res = await this.client.PATCH('/feedEntries/{id}', {
      params: {
        path: {
          id,
        },
      },
      body,
      ...options,
    })
    if (res.error) {
      if (res.response.status === 404) {
        throw new NotFoundError(res.error.message)
      }

      throw new APIError(res.error.message)
    }

    return res.data
  }
}
