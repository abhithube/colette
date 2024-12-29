import type {
  API,
  FeedEntry,
  FeedEntryListQuery,
  FeedEntryUpdate,
} from '@colette/core'
import type { MutationOptions } from '@tanstack/query-core'
import { infiniteQueryOptions } from '@tanstack/solid-query'

export const listFeedEntriesOptions = (
  query: FeedEntryListQuery,
  profileId: string,
  api: API,
) =>
  infiniteQueryOptions({
    queryKey: ['profiles', profileId, 'feedEntries', query],
    queryFn: ({ pageParam }) =>
      api.feedEntries.list({
        ...query,
        cursor: pageParam,
      }),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (lastPage) => lastPage.cursor,
  })

export type UpdateFeedEntryOptions = MutationOptions<
  FeedEntry,
  Error,
  { id: string; body: FeedEntryUpdate }
>

export const updateFeedEntryOptions = (
  options: Omit<UpdateFeedEntryOptions, 'mutationFn'>,
  api: API,
): UpdateFeedEntryOptions => ({
  ...options,
  mutationFn: (data) => api.feedEntries.update(data.id, data.body),
})
