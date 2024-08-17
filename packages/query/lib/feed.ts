import type {
  API,
  Feed,
  FeedCreate,
  FeedUpdate,
  File,
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
    queryFn: ({ signal }) =>
      api.feeds.list(query, {
        signal,
      }),
  })

export const getFeedOptions = (id: string, api: API) =>
  queryOptions({
    queryKey: ['feeds', id],
    queryFn: ({ signal }) =>
      api.feeds.get(id, {
        signal,
      }),
  })

export const createFeedOptions = (
  options: Omit<UseMutationOptions<Feed, Error, FeedCreate>, 'mutationFn'>,
  api: API,
) => {
  return {
    ...options,
    mutationFn: (body) => api.feeds.create(body),
  } as UseMutationOptions<Feed, Error, FeedCreate>
}

export const updateFeedOptions = (
  options: Omit<
    UseMutationOptions<Feed, Error, { id: string; body: FeedUpdate }>,
    'mutationFn'
  >,
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

export const importFeedsOptions = (
  options: Omit<UseMutationOptions<void, Error, File>, 'mutationFn'>,
  api: API,
) => {
  return {
    ...options,
    mutationFn: (body) => api.feeds.import(body),
  } as UseMutationOptions<void, Error, File>
}
