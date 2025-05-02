import type {
  API,
  BookmarkCreate,
  BookmarkListQuery,
  BookmarkScrape,
  BookmarkUpdate,
  LinkBookmarkTags,
} from '@colette/core'
import { useAPI } from '@colette/util'
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

export const useCreateBookmarkMutation = () => {
  const api = useAPI()
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

export const useUpdateBookmarkMutation = (id: string) => {
  const api = useAPI()
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

export const useDeleteBookmarkMutation = (id: string) => {
  const api = useAPI()
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

export const useLinkBookmarkTagsMutation = (id: string) => {
  const api = useAPI()
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

export const useScrapeBookmarkMutation = () => {
  const api = useAPI()

  return useMutation({
    mutationFn: (data: BookmarkScrape) => api.bookmarks.scrapeBookmark(data),
  })
}
