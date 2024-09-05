import type {
  API,
  Feed,
  FeedCreate,
  FeedUpdate,
  ListFeedsQuery,
} from '@colette/core'
import { type UseMutationOptions, queryOptions } from '@tanstack/react-query'

export const listFeedsOptions = (
  query: ListFeedsQuery,
  profileId: string,
  api: API,
) =>
  queryOptions({
    queryKey: ['profiles', profileId, 'feeds', query],
    queryFn: () => api.feeds.list(query),
  })

export const getFeedOptions = (id: string, api: API) =>
  queryOptions({
    queryKey: ['feeds', id],
    queryFn: () => api.feeds.get(id),
  })

export type CreateFeedOptions = UseMutationOptions<Feed, Error, FeedCreate>

export const createFeedOptions = (
  options: Omit<CreateFeedOptions, 'mutationFn'>,
  api: API,
) => {
  return {
    ...options,
    mutationFn: (body) => api.feeds.create(body),
  } as UseMutationOptions<Feed, Error, FeedCreate>
}

export type UpdateFeedOptions = UseMutationOptions<
  Feed,
  Error,
  { id: string; body: FeedUpdate }
>

export const updateFeedOptions = (
  options: Omit<UpdateFeedOptions, 'mutationFn'>,
  api: API,
) => {
  return {
    ...options,
    mutationFn: ({ id, body }) => api.feeds.update(id, body),
  } as UseMutationOptions<Feed, Error, { id: string; body: FeedUpdate }>
}

export const deleteFeedOptions = (
  id: string,
  options: Omit<UseMutationOptions, 'mutationFn'>,
  api: API,
) => {
  return {
    ...options,
    mutationFn: () => api.feeds.delete(id),
  } as UseMutationOptions
}
