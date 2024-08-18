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
    queryFn: ({ pageParam }) =>
      api.bookmarks.list({
        ...query,
        cursor: pageParam,
      }),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (lastPage) => lastPage.cursor,
  })

export type CreateBookmarkOptions = UseMutationOptions<
  Bookmark,
  Error,
  BookmarkCreate
>

export const createBookmarkOptions = (
  options: Omit<CreateBookmarkOptions, 'mutationFn'>,
  api: API,
) => {
  return {
    ...options,
    mutationFn: (body) => api.bookmarks.create(body),
  } as CreateBookmarkOptions
}

export type UpdateBookmarkOptions = UseMutationOptions<
  Bookmark,
  Error,
  { id: string; body: BookmarkUpdate }
>

export const updateBookmarkOptions = (
  options: Omit<UpdateBookmarkOptions, 'mutationFn'>,
  api: API,
) => {
  return {
    ...options,
    mutationFn: ({ id, body }) => api.bookmarks.update(id, body),
  } as UpdateBookmarkOptions
}
