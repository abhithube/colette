import {
  APIError,
  BadGatewayError,
  type Feed,
  type FeedAPI,
  type FeedCreate,
  type FeedList,
  type FeedUpdate,
  type File,
  type ListFeedsQuery,
  NotFoundError,
  type RequestOptions,
  type UUID,
  UnprocessableContentError,
  feedCreateSchema,
  feedListSchema,
  feedSchema,
  feedUpdateSchema,
  fileSchema,
  listFeedsQuerySchema,
  uuidSchema,
} from '@colette/core'
import type { Client } from './types'

export class HTTPFeedAPI implements FeedAPI {
  constructor(private client: Client) {}

  async list(
    query: ListFeedsQuery,
    options?: RequestOptions,
  ): Promise<FeedList> {
    const queryResult = await listFeedsQuerySchema.safeParseAsync(query)
    if (queryResult.error) {
      throw new UnprocessableContentError(queryResult.error.message)
    }

    const res = await this.client.GET('/feeds', {
      params: {
        query: queryResult.data,
      },
      ...options,
    })
    if (res.error) {
      throw new APIError('unknown error')
    }

    const feedListResult = await feedListSchema.safeParseAsync(res.data)
    if (feedListResult.error) {
      throw new UnprocessableContentError(feedListResult.error.message)
    }

    return feedListResult.data
  }

  async get(id: UUID, options?: RequestOptions): Promise<Feed> {
    const idResult = await uuidSchema.safeParseAsync(id)
    if (idResult.error) {
      throw new UnprocessableContentError(idResult.error.message)
    }

    const res = await this.client.GET('/feeds/{id}', {
      params: {
        path: {
          id: idResult.data,
        },
      },
      ...options,
    })
    if (res.error) {
      if (res.response.status === 404) {
        throw new NotFoundError(res.error.message)
      }

      throw new APIError(res.error.message)
    }

    const feedResult = await feedSchema.safeParseAsync(res.data)
    if (feedResult.error) {
      throw new UnprocessableContentError(feedResult.error.message)
    }

    return feedResult.data
  }

  async create(body: FeedCreate, options?: RequestOptions): Promise<Feed> {
    const bodyResult = await feedCreateSchema.safeParseAsync(body)
    if (bodyResult.error) {
      throw new UnprocessableContentError(bodyResult.error.message)
    }

    const res = await this.client.POST('/feeds', {
      body: bodyResult.data,
      ...options,
    })
    if (res.error) {
      if (res.response.status === 422) {
        throw new UnprocessableContentError(res.error.message)
      }
      if (res.response.status === 502) {
        throw new BadGatewayError(res.error.message)
      }

      throw new APIError(res.error.message)
    }

    const feedResult = await feedSchema.safeParseAsync(res.data)
    if (feedResult.error) {
      throw new UnprocessableContentError(feedResult.error.message)
    }

    return feedResult.data
  }

  async update(
    id: UUID,
    body: FeedUpdate,
    options?: RequestOptions,
  ): Promise<Feed> {
    const idResult = await uuidSchema.safeParseAsync(id)
    if (idResult.error) {
      throw new UnprocessableContentError(idResult.error.message)
    }
    const bodyResult = await feedUpdateSchema.safeParseAsync(body)
    if (bodyResult.error) {
      throw new UnprocessableContentError(bodyResult.error.message)
    }

    const res = await this.client.PATCH('/feeds/{id}', {
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
      if (res.response.status === 422) {
        throw new UnprocessableContentError(res.error.message)
      }

      throw new APIError(res.error.message)
    }

    const feedResult = await feedSchema.safeParseAsync(res.data)
    if (feedResult.error) {
      throw new UnprocessableContentError(feedResult.error.message)
    }

    return feedResult.data
  }

  async delete(id: UUID, options?: RequestOptions): Promise<void> {
    const idResult = await uuidSchema.safeParseAsync(id)
    if (idResult.error) {
      throw new UnprocessableContentError(idResult.error.message)
    }

    const res = await this.client.DELETE('/feeds/{id}', {
      params: {
        path: {
          id: idResult.data,
        },
      },
      ...options,
    })
    if (res.error) {
      if (res.response.status === 404) {
        throw new NotFoundError(res.error.message)
      }

      throw new APIError(res.error.message)
    }
  }

  async import(body: File, options?: RequestOptions): Promise<void> {
    const bodyResult = await fileSchema.safeParseAsync(body)
    if (bodyResult.error) {
      throw new UnprocessableContentError(bodyResult.error.message)
    }

    const res = await this.client.POST('/feeds/import', {
      body: bodyResult.data,
      bodySerializer: (body) => {
        const fd = new FormData()
        fd.append('file', body.data)
        return fd
      },
      ...options,
    })
    if (res.error) {
      throw new APIError('unknown error')
    }
  }
}
