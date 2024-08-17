import {
  APIError,
  BadGatewayError,
  type Bookmark,
  type BookmarkAPI,
  type BookmarkCreate,
  type BookmarkList,
  type BookmarkUpdate,
  type ListBookmarksQuery,
  NotFoundError,
  type RequestOptions,
  type UUID,
  UnprocessableContentError,
  bookmarkCreateSchema,
  bookmarkListSchema,
  bookmarkSchema,
  bookmarkUpdateSchema,
  listBookmarksQuerySchema,
  uuidSchema,
} from '@colette/core'
import type { Client } from '.'

export class HTTPBookmarkAPI implements BookmarkAPI {
  constructor(private client: Client) {}

  async list(
    query: ListBookmarksQuery,
    options?: RequestOptions,
  ): Promise<BookmarkList> {
    const queryResult = await listBookmarksQuerySchema.safeParseAsync(query)
    if (queryResult.error) {
      throw new UnprocessableContentError(queryResult.error.message)
    }

    const res = await this.client.GET('/bookmarks', {
      params: {
        query: queryResult.data,
      },
      ...options,
    })
    if (res.error) {
      throw new APIError('unknown error')
    }

    const listResult = await bookmarkListSchema.safeParseAsync(res.data)
    if (listResult.error) {
      throw new UnprocessableContentError(listResult.error.message)
    }

    return listResult.data
  }

  async get(id: UUID, options?: RequestOptions): Promise<Bookmark> {
    const idResult = await uuidSchema.safeParseAsync(id)
    if (idResult.error) {
      throw new UnprocessableContentError(idResult.error.message)
    }

    const res = await this.client.GET('/bookmarks/{id}', {
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

    const bookmarkResult = await bookmarkSchema.safeParseAsync(res.data)
    if (bookmarkResult.error) {
      throw new UnprocessableContentError(bookmarkResult.error.message)
    }

    return bookmarkResult.data
  }

  async create(
    body: BookmarkCreate,
    options?: RequestOptions,
  ): Promise<Bookmark> {
    const bodyResult = await bookmarkCreateSchema.safeParseAsync(body)
    if (bodyResult.error) {
      throw new UnprocessableContentError(bodyResult.error.message)
    }

    const res = await this.client.POST('/bookmarks', {
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

    const bookmarkResult = await bookmarkSchema.safeParseAsync(res.data)
    if (bookmarkResult.error) {
      throw new UnprocessableContentError(bookmarkResult.error.message)
    }

    return bookmarkResult.data
  }

  async update(
    id: UUID,
    body: BookmarkUpdate,
    options?: RequestOptions,
  ): Promise<Bookmark> {
    const idResult = await uuidSchema.safeParseAsync(id)
    if (idResult.error) {
      throw new UnprocessableContentError(idResult.error.message)
    }
    const bodyResult = await bookmarkUpdateSchema.safeParseAsync(body)
    if (bodyResult.error) {
      throw new UnprocessableContentError(bodyResult.error.message)
    }

    const res = await this.client.PATCH('/bookmarks/{id}', {
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

    const bookmarkResult = await bookmarkSchema.safeParseAsync(res.data)
    if (bookmarkResult.error) {
      throw new UnprocessableContentError(bookmarkResult.error.message)
    }

    return bookmarkResult.data
  }

  async delete(id: UUID, options?: RequestOptions): Promise<void> {
    const idResult = await uuidSchema.safeParseAsync(id)
    if (idResult.error) {
      throw new UnprocessableContentError(idResult.error.message)
    }

    const res = await this.client.DELETE('/bookmarks/{id}', {
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
