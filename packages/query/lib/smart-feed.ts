import type { BaseMutationOptions, BaseQueryOptions } from './common'
import type {
  API,
  SmartFeed,
  SmartFeedCreate,
  SmartFeedList,
  SmartFeedUpdate,
} from '@colette/core'
import type { QueryKey } from '@tanstack/query-core'

const SMART_FEEDS_KEY: QueryKey = ['smartFeeds']

type ListSmartFeedsOptions = BaseQueryOptions<SmartFeedList>

export const listSmartFeedsOptions = (
  api: API,
  options: Omit<ListSmartFeedsOptions, 'queryKey' | 'queryFn'> = {},
): ListSmartFeedsOptions => ({
  ...options,
  queryKey: SMART_FEEDS_KEY,
  queryFn: () => api.smartFeeds.list(),
})

type GetSmartFeedOptions = BaseQueryOptions<SmartFeed>

export const getSmartFeedOptions = (
  id: string,
  api: API,
  options: Omit<GetSmartFeedOptions, 'queryKey' | 'queryFn'> = {},
): GetSmartFeedOptions => ({
  ...options,
  queryKey: [...SMART_FEEDS_KEY, id],
  queryFn: () => api.smartFeeds.get(id),
})

type CreateSmartFeedOptions = BaseMutationOptions<SmartFeed, SmartFeedCreate>

export const createSmartFeedOptions = (
  api: API,
  options: Omit<CreateSmartFeedOptions, 'mutationFn'> = {},
): CreateSmartFeedOptions => ({
  ...options,
  mutationFn: (body) => api.smartFeeds.create(body),
})

type UpdateSmartFeedOptions = BaseMutationOptions<
  SmartFeed,
  { id: string; body: SmartFeedUpdate }
>

export const updateSmartFeedOptions = (
  api: API,
  options: Omit<UpdateSmartFeedOptions, 'mutationFn'> = {},
): UpdateSmartFeedOptions => ({
  ...options,
  mutationFn: ({ id, body }) => api.smartFeeds.update(id, body),
})

export const deleteSmartFeedOptions = (
  id: string,
  api: API,
  options: Omit<BaseMutationOptions, 'mutationFn'> = {},
): BaseMutationOptions => ({
  ...options,
  mutationFn: () => api.smartFeeds.delete(id),
})
