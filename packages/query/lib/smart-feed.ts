import type { API, SmartFeed, SmartFeedUpdate } from '@colette/core'
import { type UseMutationOptions, queryOptions } from '@tanstack/react-query'

export const listSmartFeedsOptions = (profileId: string, api: API) =>
  queryOptions({
    queryKey: ['profiles', profileId, 'smartFeeds'],
    queryFn: () => api.smartFeeds.list(),
  })

export const getSmartFeedOptions = (id: string, api: API) =>
  queryOptions({
    queryKey: ['smartFeeds', id],
    queryFn: () => api.smartFeeds.get(id),
  })

export type UpdateSmartFeedOptions = UseMutationOptions<
  SmartFeed,
  Error,
  { id: string; body: SmartFeedUpdate }
>

export const updateSmartFeedOptions = (
  options: Omit<UpdateSmartFeedOptions, 'mutationFn'>,
  api: API,
) => {
  return {
    ...options,
    mutationFn: ({ id, body }) => api.smartFeeds.update(id, body),
  } as UseMutationOptions<
    SmartFeed,
    Error,
    { id: string; body: SmartFeedUpdate }
  >
}

export const deleteSmartFeedOptions = (
  id: string,
  options: Omit<UseMutationOptions, 'mutationFn'>,
  api: API,
) => {
  return {
    ...options,
    mutationFn: () => api.smartFeeds.delete(id),
  } as UseMutationOptions
}
