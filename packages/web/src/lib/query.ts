import {
	type QueryClient,
	type UseInfiniteQueryOptions,
	infiniteQueryOptions,
} from '@tanstack/react-query'
import { client } from './client'
import type { ListEntriesQuery } from './types'

export async function ensureInfiniteQueryData(
	queryClient: QueryClient,
	options: UseInfiniteQueryOptions,
) {
	const data = queryClient.getQueryData(options.queryKey)
	if (!data) {
		await queryClient.fetchInfiniteQuery(options)
	}
}

export const listEntriesOptions = (query: ListEntriesQuery) => {
	return infiniteQueryOptions({
		queryKey: ['entries', query],
		queryFn: async ({ pageParam, signal }) => {
			const res = await client.GET('/api/v1/entries', {
				params: {
					query: {
						...query,
						publishedAt: pageParam,
					},
				},
				signal,
			})
			if (res.error) {
				throw new Error()
			}

			return res.data
		},
		initialPageParam: undefined as string | undefined,
		getNextPageParam: (lastPage, __) => {
			return lastPage.hasMore
				? lastPage.data[lastPage.data.length - 1].publishedAt
				: undefined
		},
	})
}
