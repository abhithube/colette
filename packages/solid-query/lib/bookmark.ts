import type {
  API,
  Bookmark,
  BookmarkCreate,
  BookmarkListQuery,
  BookmarkScrape,
  BookmarkScraped,
  BookmarkUpdate,
} from '@colette/core'
import type { MutationOptions } from '@tanstack/query-core'
import { infiniteQueryOptions } from '@tanstack/solid-query'

export const listBookmarksOptions = (query: BookmarkListQuery, api: API) =>
  infiniteQueryOptions({
    queryKey: ['bookmarks', query],
    queryFn: ({ pageParam }) =>
      api.bookmarks.list({
        ...query,
        cursor: pageParam,
      }),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (lastPage) => lastPage.cursor,
  })

export type CreateBookmarkOptions = MutationOptions<
  Bookmark,
  Error,
  BookmarkCreate
>

export const createBookmarkOptions = (
  options: Omit<CreateBookmarkOptions, 'mutationFn'>,
  api: API,
): CreateBookmarkOptions => ({
  ...options,
  mutationFn: (body) => api.bookmarks.create(body),
})

export type UpdateBookmarkOptions = MutationOptions<
  Bookmark,
  Error,
  { id: string; body: BookmarkUpdate }
>

export const updateBookmarkOptions = (
  options: Omit<UpdateBookmarkOptions, 'mutationFn'>,
  api: API,
): UpdateBookmarkOptions => ({
  ...options,
  mutationFn: ({ id, body }) => api.bookmarks.update(id, body),
})

export type ScrapeBookmarkOptions = MutationOptions<
  BookmarkScraped,
  Error,
  BookmarkScrape
>

export const scrapeBookmarkOptions = (
  options: Omit<ScrapeBookmarkOptions, 'mutationFn'>,
  api: API,
): ScrapeBookmarkOptions => ({
  ...options,
  mutationFn: (body) => api.bookmarks.scrape(body),
})
