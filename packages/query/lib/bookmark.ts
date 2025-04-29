import type {
  API,
  BookmarkCreate,
  BookmarkListQuery,
  BookmarkScrape,
  BookmarkUpdate,
  LinkBookmarkTags,
} from '@colette/core'
import {
  infiniteQueryOptions,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query'

const BOOKMARKS_PREFIX = 'bookmarks'

export const listBookmarksOptions = (
  api: API,
  query: Omit<BookmarkListQuery, 'cursor'> = {},
) =>
  infiniteQueryOptions({
    queryKey: [BOOKMARKS_PREFIX, query],
    queryFn: ({ pageParam }) =>
      api.bookmarks.listBookmarks({
        ...query,
        cursor: pageParam,
      }),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (lastPage) => lastPage.cursor,
  })

export const useCreateBookmarkMutation = (api: API) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: BookmarkCreate) => api.bookmarks.createBookmark(data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [BOOKMARKS_PREFIX],
      })
    },
  })
}

export const useUpdateBookmarkMutation = (api: API, id: string) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: BookmarkUpdate) =>
      api.bookmarks.updateBookmark(id, data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [BOOKMARKS_PREFIX],
      })
    },
  })
}

export const useDeleteBookmarkMutation = (api: API, id: string) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => api.bookmarks.deleteBookmark(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [BOOKMARKS_PREFIX],
      })
    },
  })
}

export const useLinkBookmarkTagsMutation = (api: API, id: string) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: LinkBookmarkTags) =>
      api.bookmarks.linkBookmarkTags(id, data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [BOOKMARKS_PREFIX],
      })
    },
  })
}

export const useScrapeBookmarkMutation = (api: API) => {
  return useMutation({
    mutationFn: (data: BookmarkScrape) => api.bookmarks.scrapeBookmark(data),
  })
}
