import type {
	QueryClient,
	UseInfiniteQueryOptions,
} from '@tanstack/react-query'

export const ensureInfiniteQueryData = async (
	queryClient: QueryClient,
	options: UseInfiniteQueryOptions,
) => {
	const data = queryClient.getQueryData(options.queryKey)
	if (!data) {
		await queryClient.fetchInfiniteQuery(options)
	}
}
