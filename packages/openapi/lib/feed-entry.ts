import {
  APIError,
  type FeedEntry,
  type FeedEntryAPI,
  type FeedEntryList,
  type FeedEntryUpdate,
  type ListFeedEntriesQuery,
  NotFoundError,
  type RequestOptions,
  type UUID,
  UnprocessableContentError,
  feedEntryListSchema,
  feedEntrySchema,
  feedEntryUpdateSchema,
  listFeedEntriesQuerySchema,
  uuidSchema,
} from '@colette/core'
import type { Client } from '.'

export class HTTPFeedEntryAPI implements FeedEntryAPI {
  constructor(private client: Client) {}

  async list(
    query: ListFeedEntriesQuery,
    options?: RequestOptions,
  ): Promise<FeedEntryList> {
    const queryResult = await listFeedEntriesQuerySchema.safeParseAsync(query)
    if (queryResult.error) {
      throw new UnprocessableContentError(queryResult.error.message)
    }

    const res = await this.client.GET('/feedEntries', {
      params: {
        query: queryResult.data,
      },
      ...options,
    })
    if (res.error) {
      throw new APIError('unknown error')
    }

    const feedEntryListResult = await feedEntryListSchema.safeParseAsync(
      res.data,
    )
    if (feedEntryListResult.error) {
      throw new UnprocessableContentError(feedEntryListResult.error.message)
    }

    return feedEntryListResult.data
  }

  async update(
    id: UUID,
    body: FeedEntryUpdate,
    options?: RequestOptions,
  ): Promise<FeedEntry> {
    const idResult = await uuidSchema.safeParseAsync(id)
    if (idResult.error) {
      throw new UnprocessableContentError(idResult.error.message)
    }
    const bodyResult = await feedEntryUpdateSchema.safeParseAsync(body)
    if (bodyResult.error) {
      throw new UnprocessableContentError(bodyResult.error.message)
    }

    const res = await this.client.PATCH('/feedEntries/{id}', {
      params: {
        path: {
          id: idResult.data,
        },
      },
      body: bodyResult.data,
      ...options,
    })
    if (res.error) {
      if (res.response.status === 404) {
        throw new NotFoundError(res.error.message)
      }

      throw new APIError(res.error.message)
    }

    const feedEntryResult = await feedEntrySchema.safeParseAsync(res.data)
    if (feedEntryResult.error) {
      throw new UnprocessableContentError(feedEntryResult.error.message)
    }

    return feedEntryResult.data
  }
}
