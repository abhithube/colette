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
import type { BaseInfiniteQueryOptions, BaseMutationOptions } from './common'

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
  options: Omit<CreateBookmarkOptions, 'mutationFn'>,
  api: API,
): CreateBookmarkOptions => ({
  ...options,
  mutationFn: (body) => api.bookmarks.create(body),
})

type UpdateBookmarkOptions = BaseMutationOptions<
  Bookmark,
  { id: string; body: BookmarkUpdate }
>

export const updateBookmarkOptions = (
  options: Omit<UpdateBookmarkOptions, 'mutationFn'>,
  api: API,
): UpdateBookmarkOptions => ({
  ...options,
  mutationFn: ({ id, body }) => api.bookmarks.update(id, body),
})

type ScrapeBookmarkOptions = BaseMutationOptions<
  BookmarkScraped,
  BookmarkScrape
>

export const scrapeBookmarkOptions = (
  options: Omit<ScrapeBookmarkOptions, 'mutationFn'>,
  api: API,
): ScrapeBookmarkOptions => ({
  ...options,
  mutationFn: (body) => api.bookmarks.scrape(body),
})
