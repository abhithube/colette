import { components, paths, operations } from './openapi'
import { Client } from 'openapi-fetch'

export type Bookmark = components['schemas']['Bookmark']
export type BookmarkDetails = components['schemas']['BookmarkDetails']
export type BookmarkDetailsList =
  components['schemas']['Paginated_BookmarkDetails']
export type BookmarkCreate = components['schemas']['BookmarkCreate']
export type BookmarkScrape = components['schemas']['BookmarkScrape']
export type BookmarkScraped = components['schemas']['BookmarkScraped']
export type BookmarkUpdate = components['schemas']['BookmarkUpdate']
export type LinkBookmarkTags = components['schemas']['LinkBookmarkTags']

export type BookmarkListQuery = NonNullable<
  operations['listBookmarks']['parameters']['query']
>
export type BookmarkGetQuery = NonNullable<
  operations['getBookmark']['parameters']['query']
>

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
  constructor(private client: Client<paths>) {}

  async listBookmarks(query: BookmarkListQuery): Promise<BookmarkDetailsList> {
    const res = await this.client.GET('/bookmarks', {
      params: {
        query,
      },
    })

    return res.data!
  }

  async getBookmark(
    id: string,
    query: BookmarkGetQuery,
  ): Promise<BookmarkDetails> {
    const res = await this.client.GET('/bookmarks/{id}', {
      params: {
        path: {
          id,
        },
        query,
      },
    })

    return res.data!
  }

  async createBookmark(data: BookmarkCreate): Promise<Bookmark> {
    const res = await this.client.POST('/bookmarks', {
      body: data,
    })

    return res.data!
  }

  async updateBookmark(id: string, data: BookmarkUpdate): Promise<Bookmark> {
    const res = await this.client.PATCH('/bookmarks/{id}', {
      params: {
        path: {
          id,
        },
      },
      body: data,
    })

    return res.data!
  }

  async deleteBookmark(id: string): Promise<void> {
    await this.client.DELETE('/bookmarks/{id}', {
      params: {
        path: {
          id,
        },
      },
    })
  }

  async linkBookmarkTags(id: string, data: LinkBookmarkTags): Promise<void> {
    await this.client.PATCH('/bookmarks/{id}/linkTags', {
      params: {
        path: {
          id,
        },
      },
      body: data,
    })
  }

  async scrapeBookmark(data: BookmarkScrape): Promise<BookmarkScraped> {
    const res = await this.client.POST('/bookmarks/scrape', {
      body: data,
    })

    return res.data!
  }
}
