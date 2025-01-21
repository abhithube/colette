import type { BaseInfiniteQueryOptions, BaseMutationOptions } from './common'
import type {
  API,
  Bookmark,
  BookmarkCreate,
  BookmarkList,
  BookmarkListQuery,
  BookmarkScrape,
  BookmarkScraped,
  BookmarkUpdate,
} from '@colette/core'
import type { QueryKey } from '@tanstack/query-core'

const BOOKMARKS_KEY: QueryKey = ['bookmarks']

type ListBookmarksOptions = BaseInfiniteQueryOptions<
  BookmarkList,
  string | undefined
>

export const listBookmarksOptions = (
  query: BookmarkListQuery,
  api: API,
  options: Omit<
    ListBookmarksOptions,
    'queryKey' | 'queryFn' | 'initialPageParam' | 'getNextPageParam'
  > = {},
): ListBookmarksOptions => ({
  ...options,
  queryKey: [...BOOKMARKS_KEY, query],
  queryFn: ({ pageParam }) =>
    api.bookmarks.list({
      ...query,
      cursor: pageParam,
    }),
  initialPageParam: undefined,
  getNextPageParam: (lastPage) => lastPage.cursor,
})

type CreateBookmarkOptions = BaseMutationOptions<Bookmark, BookmarkCreate>

export const createBookmarkOptions = (
  api: API,
  options: Omit<CreateBookmarkOptions, 'mutationFn'> = {},
): CreateBookmarkOptions => ({
  ...options,
  mutationFn: (body) => api.bookmarks.create(body),
})

type UpdateBookmarkOptions = BaseMutationOptions<
  Bookmark,
  { id: string; body: BookmarkUpdate }
>

export const updateBookmarkOptions = (
  api: API,
  options: Omit<UpdateBookmarkOptions, 'mutationFn'> = {},
): UpdateBookmarkOptions => ({
  ...options,
  mutationFn: ({ id, body }) => api.bookmarks.update(id, body),
})

type ScrapeBookmarkOptions = BaseMutationOptions<
  BookmarkScraped,
  BookmarkScrape
>

export const scrapeBookmarkOptions = (
  api: API,
  options: Omit<ScrapeBookmarkOptions, 'mutationFn'> = {},
): ScrapeBookmarkOptions => ({
  ...options,
  mutationFn: (body) => api.bookmarks.scrape(body),
})
