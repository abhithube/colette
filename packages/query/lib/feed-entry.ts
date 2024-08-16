import type {
  API,
  FeedEntry,
  FeedEntryUpdate,
  ListFeedEntriesQuery,
} from '@colette/openapi'
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
    queryFn: ({ pageParam, signal }) =>
      api.entries.list(
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

export const updateFeedEntryOptions = (
  options: Omit<
    UseMutationOptions<FeedEntry, Error, { id: string; body: FeedEntryUpdate }>,
    'mutationFn'
  >,
  api: API,
) => {
  return {
    ...options,
    mutationFn: (data) => api.entries.update(data.id, data.body),
  } as UseMutationOptions<
    FeedEntry,
    Error,
    { id: string; body: FeedEntryUpdate }
  >
}