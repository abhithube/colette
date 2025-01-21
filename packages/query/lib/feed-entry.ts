import type { BaseInfiniteQueryOptions, BaseMutationOptions } from './common'
import type {
  API,
  FeedEntry,
  FeedEntryList,
  FeedEntryListQuery,
  FeedEntryUpdate,
} from '@colette/core'
import { QueryClient } from '@tanstack/query-core'

const FEED_ENTRIES_PREFIX = 'feedEntries'

type ListFeedEntriesOptions = BaseInfiniteQueryOptions<
  FeedEntryList,
  string | undefined
>

export const listFeedEntriesOptions = (
  query: FeedEntryListQuery,
  api: API,
  options: Omit<
    ListFeedEntriesOptions,
    'queryKey' | 'queryFn' | 'initialPageParam' | 'getNextPageParam'
  > = {},
): ListFeedEntriesOptions => ({
  ...options,
  queryKey: [FEED_ENTRIES_PREFIX, query],
  queryFn: ({ pageParam }) =>
    api.feedEntries.list({
      ...query,
      cursor: pageParam,
    }),
  initialPageParam: undefined,
  getNextPageParam: (lastPage) => lastPage.cursor,
})

type UpdateFeedEntryOptions = BaseMutationOptions<
  FeedEntry,
  { id: string; body: FeedEntryUpdate }
>

export const updateFeedEntryOptions = (
  api: API,
  queryClient: QueryClient,
  options: Omit<UpdateFeedEntryOptions, 'mutationFn'> = {},
): UpdateFeedEntryOptions => ({
  ...options,
  mutationFn: (data) => api.feedEntries.update(data.id, data.body),
  onSuccess: async (...args) => {
    await queryClient.invalidateQueries({
      queryKey: [FEED_ENTRIES_PREFIX],
    })

    if (options.onSuccess) {
      await options.onSuccess(...args)
    }
  },
})
