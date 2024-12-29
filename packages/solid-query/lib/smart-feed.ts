import type {
  API,
  SmartFeed,
  SmartFeedCreate,
  SmartFeedUpdate,
} from '@colette/core'
import type { MutationOptions } from '@tanstack/query-core'
import { queryOptions } from '@tanstack/solid-query'

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

export type CreateSmartFeedOptions = MutationOptions<
  SmartFeed,
  Error,
  SmartFeedCreate
>

export const createSmartFeedOptions = (
  options: Omit<CreateSmartFeedOptions, 'mutationFn'>,
  api: API,
): CreateSmartFeedOptions => ({
  ...options,
  mutationFn: (body) => api.smartFeeds.create(body),
})

export type UpdateSmartFeedOptions = MutationOptions<
  SmartFeed,
  Error,
  { id: string; body: SmartFeedUpdate }
>

export const updateSmartFeedOptions = (
  options: Omit<UpdateSmartFeedOptions, 'mutationFn'>,
  api: API,
): UpdateSmartFeedOptions => ({
  ...options,
  mutationFn: ({ id, body }) => api.smartFeeds.update(id, body),
})

export const deleteSmartFeedOptions = (
  id: string,
  options: Omit<MutationOptions, 'mutationFn'>,
  api: API,
): MutationOptions => ({
  ...options,
  mutationFn: () => api.smartFeeds.delete(id),
})
