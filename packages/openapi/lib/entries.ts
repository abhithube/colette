import type { FetchOptions } from 'openapi-fetch'
import type { Client } from '.'
import { APIError, NotFoundError } from './error'
import type { operations } from './openapi'
import type { Entry, EntryList, EntryUpdate, ListEntriesQuery } from './types'

export class EntriesAPI {
  constructor(private client: Client) {}

  async list(
    query?: ListEntriesQuery,
    options?: Omit<FetchOptions<operations['listEntries']>, 'params'>,
  ): Promise<EntryList> {
    const res = await this.client.GET('/entries', {
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
    body: EntryUpdate,
    options?: Omit<FetchOptions<operations['updateEntry']>, 'params' | 'body'>,
  ): Promise<Entry> {
    const res = await this.client.PATCH('/entries/{id}', {
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
