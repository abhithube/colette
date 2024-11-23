import {
  Bookmark,
  type BookmarkAPI,
  BookmarkCreate,
  BookmarkList,
  BookmarkListQuery,
  BookmarkScrape,
  BookmarkScraped,
  BookmarkUpdate,
} from '@colette/core'
import { invoke } from '@tauri-apps/api/core'

export class BookmarkCommands implements BookmarkAPI {
  list(query: BookmarkListQuery): Promise<BookmarkList> {
    return invoke('list_bookmarks', {
      query: BookmarkListQuery.parse(query),
    }).then(BookmarkList.parse)
  }

  get(id: string): Promise<Bookmark> {
    return invoke('get_bookmark', { id }).then(Bookmark.parse)
  }

  create(data: BookmarkCreate): Promise<Bookmark> {
    return invoke('create_bookmark', { data: BookmarkCreate.parse(data) }).then(
      Bookmark.parse,
    )
  }

  update(id: string, data: BookmarkUpdate): Promise<Bookmark> {
    return invoke('update_bookmark', {
      id,
      data: BookmarkUpdate.parse(data),
    }).then(Bookmark.parse)
  }

  delete(id: string): Promise<void> {
    return invoke('delete_bookmark', { id }).then()
  }

  scrape(data: BookmarkScrape): Promise<BookmarkScraped> {
    return invoke('scrape_bookmark', { data: BookmarkScrape.parse(data) }).then(
      BookmarkScraped.parse,
    )
  }
}
