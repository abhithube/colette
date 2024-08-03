import type { API, Feed, FeedCreate } from '@colette/openapi'
import { type UseMutationOptions, queryOptions } from '@tanstack/react-query'

export const listFeedsOptions = (profileId: string, api: API) =>
	queryOptions({
		queryKey: ['profiles', profileId, 'feeds'],
		queryFn: ({ signal }) =>
			api.feeds.list({
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
