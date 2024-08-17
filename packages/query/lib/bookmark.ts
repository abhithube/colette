import type {
  API,
  Bookmark,
  BookmarkCreate,
  BookmarkUpdate,
  ListBookmarksQuery,
} from '@colette/core'
import {
  type UseMutationOptions,
  infiniteQueryOptions,
} from '@tanstack/react-query'

export const listBookmarksOptions = (
  query: ListBookmarksQuery,
  profileId: string,
  api: API,
) =>
  infiniteQueryOptions({
    queryKey: ['profiles', profileId, 'bookmarks', query],
    queryFn: ({ pageParam, signal }) =>
      api.bookmarks.list(
        {
          ...query,
          cursor: pageParam,
        },
        {
          signal,
        },
      ),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (lastPage) => lastPage.cursor,
  })

export const createBookmarkOptions = (
  options: Omit<
    UseMutationOptions<Bookmark, Error, BookmarkCreate>,
    'mutationFn'
  >,
  api: API,
) => {
  return {
    ...options,
    mutationFn: (body) => api.bookmarks.create(body),
  } as UseMutationOptions<Bookmark, Error, BookmarkCreate>
}

export const updateBookmarkOptions = (
  options: Omit<
    UseMutationOptions<Bookmark, Error, { id: string; body: BookmarkUpdate }>,
    'mutationFn'
  >,
  api: API,
) => {
  return {
    ...options,
    mutationFn: ({ id, body }) => api.bookmarks.update(id, body),
  } as UseMutationOptions<Bookmark, Error, { id: string; body: BookmarkUpdate }>
}
