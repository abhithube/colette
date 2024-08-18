import type {
  API,
  FeedEntry,
  FeedEntryUpdate,
  ListFeedEntriesQuery,
} from '@colette/core'
import {
  type UseMutationOptions,
  infiniteQueryOptions,
} from '@tanstack/react-query'

export const listFeedEntriesOptions = (
  query: ListFeedEntriesQuery,
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

export type UpdateFeedEntryOptions = UseMutationOptions<
  FeedEntry,
  Error,
  { id: string; body: FeedEntryUpdate }
>

export const updateFeedEntryOptions = (
  options: Omit<UpdateFeedEntryOptions, 'mutationFn'>,
  api: API,
) => {
  return {
    ...options,
    mutationFn: (data) => api.feedEntries.update(data.id, data.body),
  } as UpdateFeedEntryOptions
}
