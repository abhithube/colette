import {
  APIError,
  type ListTagsQuery,
  NotFoundError,
  type RequestOptions,
  type Tag,
  type TagAPI,
  type TagCreate,
  type TagList,
  type TagUpdate,
  type UUID,
  UnprocessableContentError,
  listTagsQuerySchema,
  tagCreateSchema,
  tagListSchema,
  tagSchema,
  tagUpdateSchema,
  uuidSchema,
} from '@colette/core'
import type { Client } from './types'

export class HTTPTagAPI implements TagAPI {
  constructor(private client: Client) {}

  async list(query: ListTagsQuery, options?: RequestOptions): Promise<TagList> {
    const queryResult = await listTagsQuerySchema.safeParseAsync(query)
    if (queryResult.error) {
      throw new UnprocessableContentError(queryResult.error.message)
    }

    const res = await this.client.GET('/tags', {
      params: {
        query: queryResult.data,
      },
      ...options,
    })
    if (res.error) {
      throw new APIError('unknown error')
    }

    const tagListResult = await tagListSchema.safeParseAsync(res.data)
    if (tagListResult.error) {
      throw new UnprocessableContentError(tagListResult.error.message)
    }

    return tagListResult.data
  }

  async get(id: UUID, options?: RequestOptions): Promise<Tag> {
    const idResult = await uuidSchema.safeParseAsync(id)
    if (idResult.error) {
      throw new UnprocessableContentError(idResult.error.message)
    }

    const res = await this.client.GET('/tags/{id}', {
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

    const tagResult = await tagSchema.safeParseAsync(res.data)
    if (tagResult.error) {
      throw new UnprocessableContentError(tagResult.error.message)
    }

    return tagResult.data
  }

  async create(body: TagCreate, options?: RequestOptions): Promise<Tag> {
    const bodyResult = await tagCreateSchema.safeParseAsync(body)
    if (bodyResult.error) {
      throw new UnprocessableContentError(bodyResult.error.message)
    }

    const res = await this.client.POST('/tags', {
      body: bodyResult.data,
      ...options,
    })
    if (res.error) {
      if (res.response.status === 422) {
        throw new UnprocessableContentError(res.error.message)
      }

      throw new APIError(res.error.message)
    }

    const tagResult = await tagSchema.safeParseAsync(res.data)
    if (tagResult.error) {
      throw new UnprocessableContentError(tagResult.error.message)
    }

    return tagResult.data
  }

  async update(
    id: UUID,
    body: TagUpdate,
    options?: RequestOptions,
  ): Promise<Tag> {
    const idResult = await uuidSchema.safeParseAsync(id)
    if (idResult.error) {
      throw new UnprocessableContentError(idResult.error.message)
    }
    const bodyResult = await tagUpdateSchema.safeParseAsync(body)
    if (bodyResult.error) {
      throw new UnprocessableContentError(bodyResult.error.message)
    }

    const res = await this.client.PATCH('/tags/{id}', {
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

    const tagResult = await tagSchema.safeParseAsync(res.data)
    if (tagResult.error) {
      throw new UnprocessableContentError(tagResult.error.message)
    }

    return tagResult.data
  }

  async delete(id: UUID, options?: RequestOptions): Promise<void> {
    const idResult = await uuidSchema.safeParseAsync(id)
    if (idResult.error) {
      throw new UnprocessableContentError(idResult.error.message)
    }

    const res = await this.client.DELETE('/tags/{id}', {
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
}
