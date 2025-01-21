import type { BaseMutationOptions, BaseQueryOptions } from './common'
import type {
  API,
  SmartFeed,
  SmartFeedCreate,
  SmartFeedList,
  SmartFeedUpdate,
} from '@colette/core'
import { QueryClient } from '@tanstack/query-core'

const SMART_FEEDS_PREFIX = 'smartFeeds'

type ListSmartFeedsOptions = BaseQueryOptions<SmartFeedList>

export const listSmartFeedsOptions = (
  api: API,
  options: Omit<ListSmartFeedsOptions, 'queryKey' | 'queryFn'> = {},
): ListSmartFeedsOptions => ({
  ...options,
  queryKey: [SMART_FEEDS_PREFIX],
  queryFn: () => api.smartFeeds.list(),
})

type GetSmartFeedOptions = BaseQueryOptions<SmartFeed>

export const getSmartFeedOptions = (
  id: string,
  api: API,
  options: Omit<GetSmartFeedOptions, 'queryKey' | 'queryFn'> = {},
): GetSmartFeedOptions => ({
  ...options,
  queryKey: [SMART_FEEDS_PREFIX, id],
  queryFn: () => api.smartFeeds.get(id),
})

type CreateSmartFeedOptions = BaseMutationOptions<SmartFeed, SmartFeedCreate>

export const createSmartFeedOptions = (
  api: API,
  queryClient: QueryClient,
  options: Omit<CreateSmartFeedOptions, 'mutationFn'> = {},
): CreateSmartFeedOptions => ({
  ...options,
  mutationFn: (body) => api.smartFeeds.create(body),
  onSuccess: async (...args) => {
    await queryClient.invalidateQueries({
      queryKey: [SMART_FEEDS_PREFIX],
    })

    if (options.onSuccess) {
      await options.onSuccess(...args)
    }
  },
})

type UpdateSmartFeedOptions = BaseMutationOptions<
  SmartFeed,
  { id: string; body: SmartFeedUpdate }
>

export const updateSmartFeedOptions = (
  api: API,
  queryClient: QueryClient,
  options: Omit<UpdateSmartFeedOptions, 'mutationFn'> = {},
): UpdateSmartFeedOptions => ({
  ...options,
  mutationFn: ({ id, body }) => api.smartFeeds.update(id, body),
  onSuccess: async (...args) => {
    await queryClient.invalidateQueries({
      queryKey: [SMART_FEEDS_PREFIX],
    })

    if (options.onSuccess) {
      await options.onSuccess(...args)
    }
  },
})

export const deleteSmartFeedOptions = (
  id: string,
  api: API,
  queryClient: QueryClient,
  options: Omit<BaseMutationOptions, 'mutationFn'> = {},
): BaseMutationOptions => ({
  ...options,
  mutationFn: () => api.smartFeeds.delete(id),
  onSuccess: async (...args) => {
    await queryClient.invalidateQueries({
      queryKey: [SMART_FEEDS_PREFIX],
    })

    if (options.onSuccess) {
      await options.onSuccess(...args)
    }
  },
})
