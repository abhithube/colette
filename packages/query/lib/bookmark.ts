import {
  BookmarkCreate,
  BookmarkScrape,
  BookmarkUpdate,
  createBookmark,
  deleteBookmark,
  linkBookmarkTags,
  LinkBookmarkTags,
  listBookmarks,
  ListBookmarksQueryParams,
  scrapeBookmark,
  updateBookmark,
} from '@colette/core'
import {
  infiniteQueryOptions,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query'

const BOOKMARKS_PREFIX = 'bookmarks'

export const listBookmarksOptions = (
  query: Omit<ListBookmarksQueryParams, 'cursor'> = {},
) =>
  infiniteQueryOptions({
    queryKey: [BOOKMARKS_PREFIX, query],
    queryFn: ({ pageParam }) =>
      listBookmarks({
        ...query,
        cursor: pageParam,
      }),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (lastPage) => lastPage.cursor,
  })

export const useCreateBookmarkMutation = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: BookmarkCreate) => createBookmark(data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [BOOKMARKS_PREFIX],
      })
    },
  })
}

export const useUpdateBookmarkMutation = (id: string) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: BookmarkUpdate) => updateBookmark(id, data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [BOOKMARKS_PREFIX],
      })
    },
  })
}

export const useDeleteBookmarkMutation = (id: string) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => deleteBookmark(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [BOOKMARKS_PREFIX],
      })
    },
  })
}

export const useLinkBookmarkTagsMutation = (id: string) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: LinkBookmarkTags) => linkBookmarkTags(id, data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [BOOKMARKS_PREFIX],
      })
    },
  })
}

export const useScrapeBookmarkMutation = () => {
  return useMutation({
    mutationFn: (data: BookmarkScrape) => scrapeBookmark(data),
  })
}
