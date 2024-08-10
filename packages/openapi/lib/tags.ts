import type { FetchOptions } from 'openapi-fetch'
import type { Client } from '.'
import { APIError, NotFoundError, UnprocessableContentError } from './error'
import type { operations } from './openapi'
import type { ListTagsQuery, Tag, TagCreate, TagList, TagUpdate } from './types'

export class TagsAPI {
  constructor(private client: Client) {}

  async list(
    query: ListTagsQuery,
    options?: FetchOptions<operations['listTags']>,
  ): Promise<TagList> {
    const res = await this.client.GET('/tags', {
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

  async get(
    id: string,
    options?: Omit<FetchOptions<operations['getTag']>, 'params'>,
  ): Promise<Tag> {
    const res = await this.client.GET('/tags/{id}', {
      params: {
        path: {
          id,
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

    return res.data
  }

  async create(
    body: TagCreate,
    options?: Omit<FetchOptions<operations['createTag']>, 'body'>,
  ): Promise<Tag> {
    const res = await this.client.POST('/tags', {
      body,
      ...options,
    })
    if (res.error) {
      if (res.response.status === 422) {
        throw new UnprocessableContentError(res.error.message)
      }

      throw new APIError(res.error.message)
    }

    return res.data
  }

  async update(
    id: string,
    body: TagUpdate,
    options?: Omit<FetchOptions<operations['updateTag']>, 'params' | 'body'>,
  ): Promise<Tag> {
    const res = await this.client.PATCH('/tags/{id}', {
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
      if (res.response.status === 422) {
        throw new UnprocessableContentError(res.error.message)
      }

      throw new APIError(res.error.message)
    }

    return res.data
  }

  async delete(
    id: string,
    options?: Omit<FetchOptions<operations['deleteTag']>, 'params'>,
  ): Promise<void> {
    const res = await this.client.DELETE('/tags/{id}', {
      params: {
        path: {
          id,
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
