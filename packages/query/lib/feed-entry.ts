import type { API, FeedEntryListQuery, FeedEntryUpdate } from '@colette/core'
import { useAPI } from '@colette/util'
import {
  infiniteQueryOptions,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query'

const FEED_ENTRIES_PREFIX = 'feedEntries'

export const listFeedEntriesOptions = (api: API, query?: FeedEntryListQuery) =>
  infiniteQueryOptions({
    queryKey: [FEED_ENTRIES_PREFIX, query],
    queryFn: ({ pageParam }) =>
      api.feedEntries.list({
        ...query,
        cursor: pageParam,
      }),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (lastPage) => lastPage.cursor,
  })

export const useUpdateFeedEntryMutation = (id: string) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: FeedEntryUpdate) => api.feedEntries.update(id, data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [FEED_ENTRIES_PREFIX],
      })
    },
  })
}
