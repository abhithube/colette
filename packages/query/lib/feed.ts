import type { BaseMutationOptions, BaseQueryOptions } from './common'
import type {
  API,
  DetectedResponse,
  Feed,
  FeedCreate,
  FeedDetect,
  FeedList,
  FeedListQuery,
  FeedUpdate,
} from '@colette/core'
import type { QueryKey } from '@tanstack/query-core'

const FEEDS_KEY: QueryKey = ['feeds']

type ListFeedsOptions = BaseQueryOptions<FeedList>

export const listFeedsOptions = (
  query: FeedListQuery,
  api: API,
  options: Omit<ListFeedsOptions, 'queryKey' | 'queryFn'> = {},
): ListFeedsOptions => ({
  ...options,
  queryKey: [...FEEDS_KEY, query],
  queryFn: () => api.feeds.list(query),
})

type GetFeedOptions = BaseQueryOptions<Feed>

export const getFeedOptions = (
  id: string,
  api: API,
  options: Omit<GetFeedOptions, 'queryKey' | 'queryFn'> = {},
): GetFeedOptions => ({
  ...options,
  queryKey: [...FEEDS_KEY, id],
  queryFn: () => api.feeds.get(id),
})

type CreateFeedOptions = BaseMutationOptions<Feed, FeedCreate>

export const createFeedOptions = (
  api: API,
  options: Omit<CreateFeedOptions, 'mutationFn'> = {},
): CreateFeedOptions => ({
  ...options,
  mutationFn: (body) => api.feeds.create(body),
})

type UpdateFeedOptions = BaseMutationOptions<
  Feed,
  { id: string; body: FeedUpdate }
>

export const updateFeedOptions = (
  api: API,
  options: Omit<UpdateFeedOptions, 'mutationFn'> = {},
): UpdateFeedOptions => ({
  ...options,
  mutationFn: ({ id, body }) => api.feeds.update(id, body),
})

export const deleteFeedOptions = (
  id: string,
  api: API,
  options: Omit<BaseMutationOptions, 'mutationFn'> = {},
): BaseMutationOptions => ({
  ...options,
  mutationFn: () => api.feeds.delete(id),
})

type DetectFeedsOptions = BaseMutationOptions<DetectedResponse, FeedDetect>

export const detectFeedsOptions = (
  api: API,
  options: Omit<DetectFeedsOptions, 'mutationFn'> = {},
): DetectFeedsOptions => ({
  ...options,
  mutationFn: (body) => api.feeds.detect(body),
})
