import type {
  API,
  Entry,
  EntryUpdate,
  ListEntriesQuery,
} from '@colette/openapi'
import {
  type UseMutationOptions,
  infiniteQueryOptions,
} from '@tanstack/react-query'

export const listEntriesOptions = (
  query: ListEntriesQuery,
  profileId: string,
  api: API,
) =>
  infiniteQueryOptions({
    queryKey: ['profiles', profileId, 'entries', query],
    queryFn: ({ pageParam, signal }) =>
      api.entries.list(
        {
          ...query,
          publishedAt: pageParam,
        },
        {
          signal,
        },
      ),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (lastPage) => {
      return lastPage.hasMore
        ? lastPage.data[lastPage.data.length - 1].publishedAt
        : undefined
    },
  })

export const updateEntryOptions = (
  options: Omit<
    UseMutationOptions<Entry, Error, { id: string; body: EntryUpdate }>,
    'mutationFn'
  >,
  api: API,
) => {
  return {
    ...options,
    mutationFn: (data) => api.entries.update(data.id, data.body),
  } as UseMutationOptions<Entry, Error, { id: string; body: EntryUpdate }>
}
