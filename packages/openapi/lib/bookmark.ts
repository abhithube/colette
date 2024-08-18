import type { z } from 'zod'
import {
  type ApiClient,
  Bookmark,
  BookmarkCreate,
  BookmarkList,
  BookmarkUpdate,
  get_ListBookmarks,
} from './openapi.gen'

const ListBookmarksQuery = get_ListBookmarks.parameters.shape.query
export type ListBookmarksQuery = z.infer<typeof ListBookmarksQuery>

export interface BookmarkAPI {
  list(query: ListBookmarksQuery): Promise<BookmarkList>

  get(id: string): Promise<Bookmark>

  create(data: BookmarkCreate): Promise<Bookmark>

  update(id: string, data: BookmarkUpdate): Promise<Bookmark>

  delete(id: string): Promise<void>
}

export class HTTPBookmarkAPI implements BookmarkAPI {
  constructor(private client: ApiClient) {}

  async list(query: ListBookmarksQuery): Promise<BookmarkList> {
    return this.client
      .get('/bookmarks', {
        query: await ListBookmarksQuery.parseAsync(query),
      })
      .then(BookmarkList.parseAsync)
  }

  async get(id: string): Promise<Bookmark> {
    return this.client
      .get('/bookmarks/{id}', {
        path: {
          id,
        },
      })
      .then(Bookmark.parseAsync)
  }

  async create(data: BookmarkCreate): Promise<Bookmark> {
    return this.client
      .post('/bookmarks', {
        body: await BookmarkCreate.parseAsync(data),
      })
      .then(Bookmark.parseAsync)
  }

  async update(id: string, data: BookmarkUpdate): Promise<Bookmark> {
    return this.client
      .patch('/bookmarks/{id}', {
        path: {
          id,
        },
        body: await BookmarkUpdate.parseAsync(data),
      })
      .then(Bookmark.parseAsync)
  }

  async delete(id: string): Promise<void> {
    return this.client
      .delete('/bookmarks/{id}', {
        path: {
          id,
        },
      })
      .then()
  }
}
