import type {
  API,
  Feed,
  FeedCreate,
  FeedDetect,
  FeedDetectedList,
  FeedListQuery,
  FeedUpdate,
} from '@colette/core'
import type { MutationOptions } from '@tanstack/query-core'
import { queryOptions } from '@tanstack/solid-query'

export const listFeedsOptions = (query: FeedListQuery, api: API) =>
  queryOptions({
    queryKey: ['feeds', query],
    queryFn: () => api.feeds.list(query),
  })

export const getFeedOptions = (id: string, api: API) =>
  queryOptions({
    queryKey: ['feeds', id],
    queryFn: () => api.feeds.get(id),
  })

export type CreateFeedOptions = MutationOptions<Feed, Error, FeedCreate>

export const createFeedOptions = (
  options: Omit<CreateFeedOptions, 'mutationFn'>,
  api: API,
): CreateFeedOptions => ({
  ...options,
  mutationFn: (body) => api.feeds.create(body),
})

export type UpdateFeedOptions = MutationOptions<
  Feed,
  Error,
  { id: string; body: FeedUpdate }
>

export const updateFeedOptions = (
  options: Omit<UpdateFeedOptions, 'mutationFn'>,
  api: API,
): UpdateFeedOptions => ({
  ...options,
  mutationFn: ({ id, body }) => api.feeds.update(id, body),
})

export const deleteFeedOptions = (
  id: string,
  options: Omit<MutationOptions, 'mutationFn'>,
  api: API,
): MutationOptions => ({
  ...options,
  mutationFn: () => api.feeds.delete(id),
})

export type DetectFeedsOptions = MutationOptions<
  FeedDetectedList,
  Error,
  FeedDetect
>

export const detectFeedsOptions = (
  options: Omit<DetectFeedsOptions, 'mutationFn'>,
  api: API,
): DetectFeedsOptions => ({
  ...options,
  mutationFn: (body) => api.feeds.detect(body),
})
