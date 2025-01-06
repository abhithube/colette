import type {
  API,
  SmartFeed,
  SmartFeedCreate,
  SmartFeedList,
  SmartFeedUpdate,
} from '@colette/core'
import type { QueryKey } from '@tanstack/query-core'
import type { BaseMutationOptions, BaseQueryOptions } from './common'

const SMART_FEEDS_KEY: QueryKey = ['smartFeeds']

type ListSmartFeedsOptions = BaseQueryOptions<SmartFeedList>

export const listSmartFeedsOptions = (
  options: Omit<ListSmartFeedsOptions, 'queryKey' | 'queryFn'>,
  api: API,
): ListSmartFeedsOptions => ({
  ...options,
  queryKey: SMART_FEEDS_KEY,
  queryFn: () => api.smartFeeds.list(),
})

type GetSmartFeedOptions = BaseQueryOptions<SmartFeed>

export const getSmartFeedOptions = (
  id: string,
  options: Omit<GetSmartFeedOptions, 'queryKey' | 'queryFn'>,
  api: API,
): GetSmartFeedOptions => ({
  ...options,
  queryKey: [...SMART_FEEDS_KEY, id],
  queryFn: () => api.smartFeeds.get(id),
})

type CreateSmartFeedOptions = BaseMutationOptions<SmartFeed, SmartFeedCreate>

export const createSmartFeedOptions = (
  options: Omit<CreateSmartFeedOptions, 'mutationFn'>,
  api: API,
): CreateSmartFeedOptions => ({
  ...options,
  mutationFn: (body) => api.smartFeeds.create(body),
})

type UpdateSmartFeedOptions = BaseMutationOptions<
  SmartFeed,
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
  options: Omit<BaseMutationOptions, 'mutationFn'>,
  api: API,
): BaseMutationOptions => ({
  ...options,
  mutationFn: () => api.smartFeeds.delete(id),
})
