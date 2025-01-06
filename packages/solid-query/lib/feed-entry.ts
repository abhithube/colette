import type {
  API,
  FeedEntry,
  FeedEntryList,
  FeedEntryListQuery,
  FeedEntryUpdate,
} from '@colette/core'
import type { QueryKey } from '@tanstack/query-core'
import type { BaseInfiniteQueryOptions, BaseMutationOptions } from './common'

const FEED_ENTRIES_KEY: QueryKey = ['feedEntries']

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
  queryKey: [...FEED_ENTRIES_KEY, query],
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
  options: Omit<UpdateFeedEntryOptions, 'mutationFn'> = {},
): UpdateFeedEntryOptions => ({
  ...options,
  mutationFn: (data) => api.feedEntries.update(data.id, data.body),
})
