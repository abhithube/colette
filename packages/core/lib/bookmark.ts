import {
  type ApiClient,
  Bookmark,
  BookmarkCreate,
  BookmarkDetails,
  BookmarkScrape,
  BookmarkScraped,
  BookmarkUpdate,
  get_GetBookmark,
  get_ListBookmarks,
  LinkBookmarkTags,
  Paginated_BookmarkDetails,
} from './openapi.gen'
import type { z } from 'zod'

export const BookmarkListQuery = get_ListBookmarks.parameters.shape.query
export type BookmarkListQuery = z.infer<typeof BookmarkListQuery>

export const BookmarkGetQuery = get_GetBookmark.parameters.shape.query
export type BookmarkGetQuery = z.infer<typeof BookmarkGetQuery>

export type BookmarkDetailsList = Paginated_BookmarkDetails
export const BookmarkDetailsList = Paginated_BookmarkDetails

export interface BookmarkAPI {
  listBookmarks(query: BookmarkListQuery): Promise<BookmarkDetailsList>

  getBookmark(id: string, query: BookmarkGetQuery): Promise<BookmarkDetails>

  createBookmark(data: BookmarkCreate): Promise<Bookmark>

  updateBookmark(id: string, data: BookmarkUpdate): Promise<Bookmark>

  deleteBookmark(id: string): Promise<void>

  linkBookmarkTags(id: string, data: LinkBookmarkTags): Promise<void>

  scrapeBookmark(data: BookmarkScrape): Promise<BookmarkScraped>
}

export class HTTPBookmarkAPI implements BookmarkAPI {
  constructor(private client: ApiClient) {}

  listBookmarks(query: BookmarkListQuery): Promise<BookmarkDetailsList> {
    return this.client
      .get('/bookmarks', {
        query: BookmarkListQuery.parse(query),
      })
      .then(BookmarkDetailsList.parse)
  }

  getBookmark(id: string, query: BookmarkGetQuery): Promise<BookmarkDetails> {
    return this.client
      .get('/bookmarks/{id}', {
        path: {
          id,
        },
        query: BookmarkGetQuery.parse(query),
      })
      .then(BookmarkDetails.parse)
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

  linkBookmarkTags(id: string, data: LinkBookmarkTags): Promise<void> {
    return this.client
      .patch('/bookmarks/{id}/linkTags', {
        path: {
          id,
        },
        body: LinkBookmarkTags.parse(data),
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
