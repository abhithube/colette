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
import type { QueryClient } from '@tanstack/query-core'

const BOOKMARKS_PREFIX = 'bookmarks'

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
  queryKey: [BOOKMARKS_PREFIX, query],
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
  queryClient: QueryClient,
  options: Omit<CreateBookmarkOptions, 'mutationFn'> = {},
): CreateBookmarkOptions => ({
  ...options,
  mutationFn: (body) => api.bookmarks.create(body),
  onSuccess: async (...args) => {
    await queryClient.invalidateQueries({
      queryKey: [BOOKMARKS_PREFIX],
    })

    if (options.onSuccess) {
      await options.onSuccess(...args)
    }
  },
})

type UpdateBookmarkOptions = BaseMutationOptions<
  Bookmark,
  { id: string; body: BookmarkUpdate }
>

export const updateBookmarkOptions = (
  api: API,
  queryClient: QueryClient,
  options: Omit<UpdateBookmarkOptions, 'mutationFn'> = {},
): UpdateBookmarkOptions => ({
  ...options,
  mutationFn: ({ id, body }) => api.bookmarks.update(id, body),
  onSuccess: async (...args) => {
    await queryClient.invalidateQueries({
      queryKey: [BOOKMARKS_PREFIX],
    })

    if (options.onSuccess) {
      await options.onSuccess(...args)
    }
  },
})

export const deleteBookmarkOptions = (
  id: string,
  api: API,
  queryClient: QueryClient,
  options: Omit<BaseMutationOptions, 'mutationFn'> = {},
): BaseMutationOptions => ({
  ...options,
  mutationFn: () => api.bookmarks.delete(id),
  onSuccess: async (...args) => {
    await queryClient.invalidateQueries({
      queryKey: [BOOKMARKS_PREFIX],
    })

    if (options.onSuccess) {
      await options.onSuccess(...args)
    }
  },
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
