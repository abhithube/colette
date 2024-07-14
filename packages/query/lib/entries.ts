import type { API, ListEntriesQuery } from '@colette/openapi'
import { infiniteQueryOptions } from '@tanstack/react-query'

export const listEntriesOptions = (
	query: ListEntriesQuery,
	profileId: string,
	api: API,
) =>
	infiniteQueryOptions({
		queryKey: ['profiles', profileId, 'entries', query],
		queryFn: ({ pageParam, signal }) =>
			api.entries.list(
				{
					...query,
					publishedAt: pageParam,
				},
				{
					signal,
				},
			),
		initialPageParam: undefined as string | undefined,
		getNextPageParam: (lastPage) => {
			return lastPage.hasMore
				? lastPage.data[lastPage.data.length - 1].publishedAt
				: undefined
		},
	})
