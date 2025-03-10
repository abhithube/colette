import {
  type ApiClient,
  Bookmark,
  BookmarkCreate,
  BookmarkScrape,
  BookmarkScraped,
  BookmarkUpdate,
  Paginated_Bookmark,
  get_ListBookmarks,
} from './openapi.gen'
import type { z } from 'zod'

export const BookmarkListQuery = get_ListBookmarks.parameters.shape.query
export type BookmarkListQuery = z.infer<typeof BookmarkListQuery>

export type BookmarkList = Paginated_Bookmark
export const BookmarkList = Paginated_Bookmark

export interface BookmarkAPI {
  listBookmarks(query: BookmarkListQuery): Promise<BookmarkList>

  getBookmark(id: string): Promise<Bookmark>

  createBookmark(data: BookmarkCreate): Promise<Bookmark>

  updateBookmark(id: string, data: BookmarkUpdate): Promise<Bookmark>

  deleteBookmark(id: string): Promise<void>

  scrapeBookmark(data: BookmarkScrape): Promise<BookmarkScraped>
}

export class HTTPBookmarkAPI implements BookmarkAPI {
  constructor(private client: ApiClient) {}

  listBookmarks(query: BookmarkListQuery): Promise<BookmarkList> {
    return this.client
      .get('/bookmarks', {
        query: BookmarkListQuery.parse(query),
      })
      .then(BookmarkList.parse)
  }

  getBookmark(id: string): Promise<Bookmark> {
    return this.client
      .get('/bookmarks/{id}', {
        path: {
          id,
        },
      })
      .then(Bookmark.parse)
  }

  createBookmark(data: BookmarkCreate): Promise<Bookmark> {
    return this.client
      .post('/bookmarks', {
        body: BookmarkCreate.parse(data),
      })
      .then(Bookmark.parse)
  }

  updateBookmark(id: string, data: BookmarkUpdate): Promise<Bookmark> {
    return this.client
      .patch('/bookmarks/{id}', {
        path: {
          id,
        },
        body: BookmarkUpdate.parse(data),
      })
      .then(Bookmark.parse)
  }

  deleteBookmark(id: string): Promise<void> {
    return this.client
      .delete('/bookmarks/{id}', {
        path: {
          id,
        },
      })
      .then()
  }

  scrapeBookmark(data: BookmarkScrape): Promise<BookmarkScraped> {
    return this.client
      .post('/bookmarks/scrape', {
        body: BookmarkScrape.parse(data),
      })
      .then(BookmarkScraped.parse)
  }
}
